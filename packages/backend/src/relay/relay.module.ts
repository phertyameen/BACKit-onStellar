import { Module } from '@nestjs/common';
import { SorobanRpc } from '@stellar/stellar-sdk';
import { RelayController } from './relay.controller';
import { RelayService } from './relay.service';
import { PlatformConfigModule } from '../config/config.module';

@Module({
  imports: [PlatformConfigModule],
  controllers: [RelayController],
  providers: [
    RelayService,
    {
      provide: SorobanRpc.Server,
      useFactory: () => {
        return new SorobanRpc.Server(
          process.env.SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org',
        );
      },
    },
  ],
})
export class RelayModule {}
