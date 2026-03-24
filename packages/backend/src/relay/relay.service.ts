import { Injectable, Logger, BadRequestException } from '@nestjs/common';
import { 
  SorobanRpc, 
  Transaction, 
  FeeBumpTransaction, 
  TransactionBuilder, 
  Keypair, 
  Networks, 
  xdr,
  StrKey,
} from '@stellar/stellar-sdk';
import { ConfigService } from '../config/config.service';

@Injectable()
export class RelayService {
  private readonly logger = new Logger(RelayService.name);
  private readonly hotWallet: Keypair;

  constructor(
    private readonly configService: ConfigService,
    private readonly rpcServer: SorobanRpc.Server,
  ) {
    const secret = process.env.RELAY_HOT_WALLET_SECRET;
    if (!secret) {
      this.logger.warn('RELAY_HOT_WALLET_SECRET not set. Relay will not work.');
      // For now, don't throw to avoid crashing the app if not used
    } else {
      this.hotWallet = Keypair.fromSecret(secret);
      this.logger.log(`Relay active with sponsor address: ${this.hotWallet.publicKey()}`);
    }
  }

  async sponsorAndSubmit(xdrString: string): Promise<{ hash: string }> {
    if (!this.hotWallet) {
      throw new BadRequestException('Relay not configured (missing secret key)');
    }

    const networkPassphrase = process.env.NETWORK_PASSPHRASE || Networks.TESTNET;
    let tx: Transaction | FeeBumpTransaction;

    try {
      tx = TransactionBuilder.fromXDR(xdrString, networkPassphrase) as Transaction | FeeBumpTransaction;
    } catch (error) {
      throw new BadRequestException('Invalid XDR');
    }

    // Determine if it's already a fee-bump or a regular transaction
    const innerTx = tx instanceof FeeBumpTransaction ? tx.innerTransaction : tx;

    // Validate inner transaction
    await this.validateTransaction(innerTx);

    // If it's not a fee-bump, we wrap it in a fee-bump
    // Actually, if the user sent a regular transaction, we create a fee-bump for it.
    // The user should have signed the inner transaction already.
    
    let finalTx: FeeBumpTransaction;
    if (tx instanceof FeeBumpTransaction) {
      // If they already sent a fee-bump, they expect us to sign it?
      // But they'd need to know our public key to set it as fee source.
      // Usually, the relayer creates the fee-bump.
      finalTx = tx;
      // We should check if the fee source is us
      if (finalTx.feeSource !== this.hotWallet.publicKey()) {
        throw new BadRequestException('Fee source in fee-bump transaction must be the relayer');
      }
    } else {
      // Create a fee-bump transaction
      finalTx = TransactionBuilder.buildFeeBumpTransaction(
        this.hotWallet,
        innerTx.fee, // Or we can compute a higher fee if needed
        innerTx,
        networkPassphrase,
      );
    }

    // Sign with relayer hot wallet
    finalTx.sign(this.hotWallet);

    // Submit back to Stellar
    try {
      const response = await this.rpcServer.sendTransaction(finalTx);
      if (response.status === 'ERROR') {
          this.logger.error(`Transaction failed: ${JSON.stringify(response.errorResult)}`);
          throw new BadRequestException('Transaction submission failed');
      }
      return { hash: response.hash };
    } catch (error) {
      this.logger.error(`Relay submission error: ${error.message}`);
      throw new BadRequestException(`Submission failed: ${error.message}`);
    }
  }

  private async validateTransaction(tx: Transaction): Promise<void> {
    const settings = await this.configService.getSettings();
    const allowedContractId = settings.contractId;

    if (!allowedContractId) {
      throw new BadRequestException('Target contract not configured');
    }

    // Check all operations for contract ID
    for (const op of tx.operations) {
      if (op.type !== 'invokeHostFunction') {
        throw new BadRequestException(`Operation type ${op.type} not allowed in relay`);
      }

      // Extract the HostFunction from the high-level operation object
      const hostFunction = (op as any).func as xdr.HostFunction;
      
      if (!hostFunction) {
          throw new BadRequestException('Malformed host function operation');
      }
      
      if (hostFunction.switch().value !== xdr.HostFunctionType.hostFunctionTypeInvokeContract().value) {
          throw new BadRequestException('Only contract invocations are allowed');
      }

      const invokeContractArgs = hostFunction.invokeContract();
      const contractAddress = invokeContractArgs.contractAddress();
      
      let contractId: string;
      if (contractAddress.switch().value === xdr.ScAddressType.scAddressTypeContract().value) {
          contractId = StrKey.encodeContract(contractAddress.contractId());
      } else {
          throw new BadRequestException('Invalid contract address type');
      }

      if (contractId !== allowedContractId) {
        throw new BadRequestException(`Transaction directed at unauthorized contract: ${contractId}`);
      }
    }
  }
}
