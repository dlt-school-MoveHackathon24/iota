import { Injectable } from '@nestjs/common';

export interface Account {
  alias: string;
  address: string;
  balance: number;
}

@Injectable()
export class AccountService {
  private accounts: Account[] = [];

  // Aggiungi un account alla lista
  addAccount(alias: string, address: string, balance: number): string {
    const account: Account = { alias, address, balance };
    this.accounts.push(account);
    return `Account ${alias} aggiunto con successo.`;
  }

  // Ottieni tutti gli account
  getAccounts(): Account[] {
    return this.accounts;
  }

  // Ottieni un account tramite alias
  getAccountByAlias(alias: string): Account | string {
    const account = this.accounts.find(acc => acc.alias === alias);
    if (!account) {
      return `Nessun account trovato con alias ${alias}`;
    }
    return account;
  }
}
