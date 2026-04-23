<p align="center">
  <a href="http://nestjs.com/" target="blank"><img src="https://nestjs.com/img/logo-small.svg" width="120" alt="Nest Logo" /></a>
</p>

[circleci-image]: https://img.shields.io/circleci/build/github/nestjs/nest/master?token=abc123def456
[circleci-url]: https://circleci.com/gh/nestjs/nest

  <p align="center">A progressive <a href="http://nodejs.org" target="_blank">Node.js</a> framework for building efficient and scalable server-side applications.</p>
    <p align="center">
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/v/@nestjs/core.svg" alt="NPM Version" /></a>
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/l/@nestjs/core.svg" alt="Package License" /></a>
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/dm/@nestjs/common.svg" alt="NPM Downloads" /></a>
<a href="https://circleci.com/gh/nestjs/nest" target="_blank"><img src="https://img.shields.io/circleci/build/github/nestjs/nest/master" alt="CircleCI" /></a>
<a href="https://discord.gg/G7Qnnhy" target="_blank"><img src="https://img.shields.io/badge/discord-online-brightgreen.svg" alt="Discord"/></a>
<a href="https://opencollective.com/nest#backer" target="_blank"><img src="https://opencollective.com/nest/backers/badge.svg" alt="Backers on Open Collective" /></a>
<a href="https://opencollective.com/nest#sponsor" target="_blank"><img src="https://opencollective.com/nest/sponsors/badge.svg" alt="Sponsors on Open Collective" /></a>
  <a href="https://paypal.me/kamilmysliwiec" target="_blank"><img src="https://img.shields.io/badge/Donate-PayPal-ff3f59.svg" alt="Donate us"/></a>
    <a href="https://opencollective.com/nest#sponsor"  target="_blank"><img src="https://img.shields.io/badge/Support%20us-Open%20Collective-41B883.svg" alt="Support us"></a>
  <a href="https://twitter.com/nestframework" target="_blank"><img src="https://img.shields.io/twitter/follow/nestframework.svg?style=social&label=Follow" alt="Follow us on Twitter"></a>
</p>
  <!--[![Backers on Open Collective](https://opencollective.com/nest/backers/badge.svg)](https://opencollective.com/nest#backer)
  [![Sponsors on Open Collective](https://opencollective.com/nest/sponsors/badge.svg)](https://opencollective.com/nest#sponsor)-->

## Description

[Nest](https://github.com/nestjs/nest) framework TypeScript starter repository.

## Project setup

```bash
$ pnpm install
```

## Compile and run the project

```bash
# development
$ pnpm run start

# watch mode
$ pnpm run start:dev

# production mode
$ pnpm run start:prod
```

## Run tests

```bash
# unit tests
$ pnpm run test

# e2e tests
$ pnpm run test:e2e

# test coverage
$ pnpm run test:cov
```

# WebSocket Events Gateway

Location: `packages/backend/src/gateways/`

## Architecture

```
Client (browser / mobile)
   │  Socket.io ws://api/events
   │
   ▼
EventsGateway  (/events namespace)
   │
   ├─ handleConnection()       → auto-joins user:<id> room if JWT present
   ├─ subscribeMarket          → joins  market:<marketId> room
   ├─ unsubscribeMarket        → leaves market:<marketId> room
   └─ authenticate             → mid-session login → joins user:<id> room
   │
   └─ @OnEvent() listeners ←── EventEmitter2 (from service layer / Issue 9)
         stake.created         → broadcast → market:<id>
         price.updated         → broadcast → market:<id>
         outcome.proposed      → broadcast → market:<id>
         dispute.raised        → broadcast → market:<id> + user:<staker>
         dispute.resolved      → broadcast → market:<id> + user:<staker>
         user.notification     → send     → user:<id>
```

## Installation

```bash
npm install @nestjs/websockets @nestjs/platform-socket.io socket.io @nestjs/event-emitter
```

## AppModule wiring

See `app.module.snippet.ts`. The two required additions are:
1. `EventEmitterModule.forRoot({ wildcard: true, delimiter: '.' })`
2. `GatewaysModule`

## Client usage

### Connect (anonymous — public market data only)

```typescript
import { io } from 'socket.io-client';

const socket = io('wss://api.example.com/events', {
  transports: ['websocket'],
});

// Subscribe to a market
socket.emit('subscribeMarket', { marketId: 'mkt-abc' });

// Receive live events
socket.on('stakeCreated',    (data) => console.log('new stake', data));
socket.on('priceUpdated',    (data) => console.log('price',     data));
socket.on('outcomeProposed', (data) => console.log('outcome',   data));
socket.on('disputeRaised',   (data) => console.log('dispute',   data));
socket.on('disputeResolved', (data) => console.log('resolved',  data));
```

### Connect (authenticated — private notifications)

**Option A** — JWT in Authorization header at connection time:

```typescript
const socket = io('wss://api.example.com/events', {
  transports: ['websocket'],
  extraHeaders: { Authorization: `Bearer ${token}` },
});
```

**Option B** — Authenticate after connecting (SPA login flow):

```typescript
socket.emit('authenticate', { token: jwtToken });
socket.on('authenticated', ({ userId }) => console.log('logged in as', userId));
```

Private notifications arrive on the `notification` event:

```typescript
socket.on('notification', ({ type, payload, timestamp }) => {
  console.log(`[${type}]`, payload);
});
```

## Emitting events from your service layer

Inject `EventEmitter2` and emit with the dot-delimited keys the gateway listens to:

```typescript
import { Injectable } from '@nestjs/common';
import { EventEmitter2 } from '@nestjs/event-emitter';
import { StakeCreatedEvent } from '../gateways/events.types';

@Injectable()
export class StakesService {
  constructor(private readonly emitter: EventEmitter2) {}

  async createStake(...) {
    // ... business logic ...

    this.emitter.emit('stake.created', {
      marketId: stake.marketId,
      staker: stake.stakerAddress,
      amount: stake.amount.toString(),
      outcomeIndex: stake.outcomeIndex,
      timestamp: Date.now(),
      txHash: tx.hash,
    } satisfies StakeCreatedEvent);
  }
}
```

## Event payload reference

| EventEmitter2 key   | Socket.io client event | Room(s) notified                      |
|---------------------|------------------------|---------------------------------------|
| `stake.created`     | `stakeCreated`         | `market:<id>`                         |
| `price.updated`     | `priceUpdated`         | `market:<id>`                         |
| `outcome.proposed`  | `outcomeProposed`      | `market:<id>`                         |
| `dispute.raised`    | `disputeRaised`        | `market:<id>` + `user:<staker>`       |
| `dispute.resolved`  | `disputeResolved`      | `market:<id>` + `user:<staker>`       |
| `user.notification` | `notification`         | `user:<id>`                           |

## Imperative broadcasts (inject EventsGateway directly)

```typescript
constructor(private readonly eventsGateway: EventsGateway) {}

// Broadcast to all clients watching a market
this.eventsGateway.broadcastToMarket(marketId, 'customEvent', payload);

// Push to a single user
this.eventsGateway.sendToUser(userId, 'notification', payload);
```