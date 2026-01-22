import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { CallsModule } from '../calls/calls.module';

@Module({
  imports: [
    TypeOrmModule.forRoot({
      type: 'postgres',
      host: process.env.DB_HOST || 'localhost',
      port: parseInt(process.env.DB_PORT, 10) || 5432,
      username: process.env.DB_USERNAME || 'postgres',
      password: process.env.DB_PASSWORD || 'postgres',
      database: process.env.DB_NAME || 'backit',
      autoLoadEntities: true,
      synchronize: process.env.NODE_ENV !== 'production', // Only sync in development
    }),
    CallsModule,
  ],
  controllers: [],
  providers: [],
})
export class AppModule {}