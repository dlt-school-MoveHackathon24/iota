// src/utils/iota-object.service.ts

import { Injectable } from '@nestjs/common';

@Injectable()
export class IotaObjectService {
  // Un mapping tra l'indirizzo di un fixed account e una lista di oggetti IOTA (indirizzi)
  private iotaObjects: { [accountAddress: string]: string[] } = {
    '0x2b2517ff41d1904db3884ed87377142b88949acbcfa320fe9492cc477b314cc3': [
      '0x570d8e0ac97ff7007d8e1026127c6b505631d17468e3d4172b5ab12d47abba20',
      '0x5a259835e4e9c26a7abb14454fda5cc9c623e78ac66af4c85dc95e220e0b731c',

      '0x5e0e94ba30f11a22ce8e2b251cb86a3560fd6a286d14f3e06b28ca3a0a77d322',
      '0xb7972d1e9047fc55a98cfae1eead828bd15427e4827ec8da1806c523c8376149',
      '0xef44676f362fe91d3ebc380be9b7c14e9f8aebffb481e8cbb23f34059b916d3b',
      '0xf24cfad127dc4fa3ae169b9222351d520f4a170a6eea64108a981646cf9dc358',
      '0xfce3b0f3f604b5668e8539e17d1bd951ed250a49cb0c232c23c2f680d2d707fc'
    ]
  };

  // Metodo per ottenere gli oggetti IOTA associati a un fixed account
  getIotaObjectsForAccount(accountAddress: string): string[] {
    return this.iotaObjects[accountAddress] || [];
  }
}
