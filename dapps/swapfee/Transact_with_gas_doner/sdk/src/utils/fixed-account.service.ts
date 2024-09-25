// src/utils/fixed-account.service.ts

import { Injectable } from '@nestjs/common';
import { FixedAccount } from './fixed-account.model';


//idObjectToTrasnfer

@Injectable()
export class FixedAccountService {
  private fixedAccounts: FixedAccount[] = [
    new FixedAccount('Add1Sponsor', '0x2b2517ff41d1904db3884ed87377142b88949acbcfa320fe9492cc477b314cc3', 10000000000,0),
    new FixedAccount('Add2Sender', '0x24709573ca85741a12b645837e2e11c81e4c857a6e62a24d2bb85576b4192856', 0,1000),
    new FixedAccount('Add2Receiver', '0x10c6fc9e944a6895738a547ded7ff1c69cc7f277829f8630693d4f469541c272', 0,1000),

    //new FixedAccount('Add3', '0xbf34aede07dd209adffe79cfb3c64f044aef1ea3ffe617a618093903baa3ce56', 0,10000000000),
    new FixedAccount('Add3Sponsor', '0xbf34aede07dd209adffe79cfb3c64f044aef1ea3ffe617a618093903baa3ce56', 0,10000000000),

    ];

  getFixedAccounts(): FixedAccount[] {
    return this.fixedAccounts;
  }
}
