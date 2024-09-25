import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { CurrencyConverterService } from './utils/currency-converter.service';
import { AccountService } from './utils/account.service'; // Importa AccountService
import { FixedAccountService } from './utils/fixed-account.service';
import { IotaObjectService } from './utils/iota-object.service'; // Importa il nuovo servizio


@Module({
  imports: [],
  controllers: [AppController],
  providers: [AppService,CurrencyConverterService, AccountService, FixedAccountService, IotaObjectService],
})
export class AppModule {}
