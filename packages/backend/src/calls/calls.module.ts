import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { CallsController } from './calls.controller';
import { CallsService } from './calls.service';
import { CallEntity } from './calls.entity';
import { IpfsService } from '../storage/ipfs.service';

@Module({
  imports: [TypeOrmModule.forFeature([CallEntity])],
  controllers: [CallsController],
  providers: [CallsService, IpfsService],
  exports: [CallsService],
})
export class CallsModule {}