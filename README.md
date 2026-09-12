# Rust WebRTC example with mediasoup

This repository is a sample implementation of a multi-user WebRTC application. It demonstrates how to build the server side of an SFU-based audio/video room using Rust, Actix, and mediasoup.

It is intended as reference code for learning and experimentation, not as a reusable framework or a production-ready conferencing service.

The companion frontend implementation is expected next to this repository at:

```text
../fe
```

The two repositories form one example application:

```text
webrtc/
├── be/  Rust API, WebSocket signalling, room actors, and mediasoup SFU
└── fe/  React demo client using mediasoup-client
```

## What this example implements

The backend currently demonstrates:

- User registration and JWT authentication.
- Room creation, discovery, and membership.
- WebSocket-based signalling.
- mediasoup workers and one router per active room.
- Separate producer and consumer WebRTC transports for each peer.
- Audio, camera, and screen-share producers.
- Subscription to remote producers through mediasoup consumers.
- Producer pause, resume, and close operations.
- In-room messages and participant updates.
- Basic room-owner moderation.
- MongoDB persistence for application data.
- Redis caching for conversation lookups and short-lived data.

The companion frontend is a real consumer of this protocol. Its mediasoup integration is implemented in:

```text
../fe/apps/forum-ui/src/socket-client/SocketBinary.ts
```

The frontend uses React, Nx, Redux, and `mediasoup-client@3.7.18`. It includes room creation and joining, microphone/camera controls, screen sharing, remote media subscription, room chat, and moderation UI.

## Architecture

This example uses an SFU rather than a peer-to-peer mesh.

```text
                        MongoDB
                           ^
                           |
React client ---- REST ----+---- Actix Web
     |                             |
     +------ WebSocket signalling -+
                                   |
                              Session actor
                                   |
                                Peer actor
                                   |
                               Rooms actor
                                   |
                         one Room actor per room
                                   |
                           mediasoup Router
                         /                  \
              producer transports     consumer transports
                  client -> SFU           SFU -> clients
```

Each participant sends its media to mediasoup once. mediasoup then forwards that media to the consumers created by the other participants. This avoids the upload cost of a full peer-to-peer mesh and provides a useful base for multi-user rooms.

Actix actors keep the live WebRTC state in memory:

- `Session` represents one authenticated WebSocket connection.
- `PeerActor` tracks connected users and their sessions.
- `RoomsActor` assigns mediasoup workers and stores active room actors.
- `RoomActor` owns the live transports, producers, and consumers for a room.

MongoDB stores persistent application entities such as users, rooms, conversations, and messages. Live transports and media streams are not persistent and must be re-established after a process restart.

## Media and signalling flow

The frontend and backend use the following flow:

1. The frontend registers or logs in through the REST API and stores the returned JWT.
2. It opens `/ws`, passing the JWT as the WebSocket subprotocol.
3. It sends `JoinRoomRequest` with the room code.
4. The backend locates the room actor and returns `JoinRoomRespond` with the router RTP capabilities.
5. The frontend loads those capabilities into a mediasoup-client `Device`.
6. It sends `CreateTransportRequest`.
7. The backend creates a send transport and a receive transport and returns both in `CreateTransportRespond`.
8. mediasoup-client emits `connect`; the frontend forwards the DTLS parameters through `ConnectProducerTransportRequest` or `ConnectConsumerTransportRequest`.
9. When the user enables a microphone, camera, or screen share, mediasoup-client emits `produce`. The frontend sends `NewProducerRequest` with the media kind and RTP parameters.
10. The backend creates the producer and broadcasts `RoomMediasRespond`, containing the producers currently available in the room.
11. The frontend requests a consumer for each remote producer with `NewConsumerRequest`.
12. The backend returns `CreateConsumerRespond`; the frontend creates the local consumer and attaches its track to an HTML media element.

The signalling messages are JSON. Client requests are sent as WebSocket text frames. Server events are JSON encoded into WebSocket binary frames, which the frontend decodes with `TextDecoder`.

## TURN status

TURN is not implemented yet.

The current example advertises the mediasoup WebRTC server directly and is suitable for local development or networks where clients can reach the announced address and UDP port range. A later TURN implementation will provide a relay fallback for clients behind restrictive NATs, corporate firewalls, or networks that block direct UDP connectivity.

TURN complements mediasoup; it does not replace the SFU. The intended final path is:

```text
client -- direct ICE/UDP when possible --> mediasoup
client -- TURN relay when required -----> mediasoup
```

For deployment before TURN is added, `MEDIASOUP_ANNOUNCED_IP` must be reachable by clients and UDP ports `10000-10100` must be allowed through the firewall and NAT.

## Run the complete example

### 1. Start MongoDB and Redis

From this backend repository:

```bash
docker compose up -d
docker compose ps
```

The Compose file starts:

- MongoDB on `localhost:27017`.
- Redis on `localhost:6378`.
- Persistent named volumes for both services.
- Health checks for both services.

### 2. Start the backend

```bash
cp .env.example .env
cargo run
```

The backend starts at `http://127.0.0.1:8000`.

### 3. Start the companion frontend

In a second terminal:

```bash
cd ../fe
cp apps/forum-ui/.env.sample apps/forum-ui/.env
pnpm install
pnpm fe
```

The Vite development server starts at `http://localhost:5201`.

The frontend environment should point to this backend:

```dotenv
VITE_API_HOST=http://127.0.0.1:8000
VITE_API_WS=ws://127.0.0.1:8000
```

Open two browser sessions, create two accounts, join the same room, and enable the microphone or camera to exercise the complete producer/consumer flow.

Browser media APIs generally require `localhost` or HTTPS. A remote deployment should expose both applications over HTTPS and use WSS for signalling.

## Backend configuration

The defaults in `.env.example` match `docker-compose.yml`.

| Variable | Purpose | Local example |
| --- | --- | --- |
| `API_PORT` | REST and WebSocket port | `8000` |
| `APP_NAME` | Application identifier | `R_CHAT` |
| `JWT_SECRET_KEY` | JWT signing secret | replace the sample value |
| `HASH_ROUND` | bcrypt cost | `12` |
| `DB_CONNECTION_STRING` | MongoDB connection string | see `.env.example` |
| `DATABASE_NAME` | MongoDB database | `webrtc` |
| `REDIS_URL` | Redis connection URL | see `.env.example` |
| `MEDIASOUP_LISTEN_IP` | Local address used by mediasoup | `127.0.0.1` |
| `MEDIASOUP_ANNOUNCED_IP` | Address advertised to clients | `127.0.0.1` |
| `NUM_WORKER` | Number of mediasoup workers | `2` |
| `RTC_MIN_PORT` | Intended first UDP media port | `10000` |
| `RTC_MAX_PORT` | Intended last UDP media port | `10100` |

Two implementation details are worth noting:

- The mediasoup UDP range is currently hard-coded to `10000-10100` in `src/core/bootstrap.rs`; the `RTC_*` variables are declared but not yet used there.
- Keep `NUM_WORKER >= 2`. The current round-robin selector starts at worker index `1`, while workers are created starting at index `0`, so a value of `1` prevents a room from obtaining a worker.

## WebSocket protocol

Connect by passing the access token as the subprotocol, without the `Bearer` prefix:

```ts
const socket = new WebSocket('ws://127.0.0.1:8000/ws', [accessToken]);
socket.binaryType = 'arraybuffer';
```

Messages sent by the client use this envelope:

```json
{
  "event": "JoinRoomRequest",
  "data": {
    "roomCode": "example-room"
  }
}
```

Supported room/media events:

| Client event | Data | Backend event |
| --- | --- | --- |
| `JoinRoomRequest` | `{ roomCode }` | `JoinRoomRespond` |
| `CreateTransportRequest` | omitted | `CreateTransportRespond` |
| `ConnectProducerTransportRequest` | DTLS parameters | none |
| `ConnectConsumerTransportRequest` | DTLS parameters | none |
| `NewProducerRequest` | `{ kind, rtpParameters }` | `RoomMediasRespond` |
| `NewConsumerRequest` | `{ producerId, rtpCapabilities }` | `CreateConsumerRespond` |
| `PauseProducer` | `{ producerId }` | `RoomMediasRespond` |
| `ResumeProducer` | `{ producerId }` | `RoomMediasRespond` |
| `CloseProducer` | `{ producerId }` | `RoomMediasRespond` |
| `NewRoomMessage` | `{ text?, gif? }` | `NewRoomMessageRespond` |
| `BannedUserOutRoom` | `{ userCode }` | `RoomBannedRespond` |
| `PeerLeaveRoom` | omitted | updated room state |
| `Ping` | omitted | `Pong` text frame |

The implementation is intentionally simple. `NewProducerRequest` currently has no correlated acknowledgement containing the producer ID. The frontend works around this by passing the transport ID to the mediasoup-client callback and later reading the real producer ID from `RoomMediasRespond`. A stronger implementation should add a `CreateProducerRespond` event and call the frontend callback only after receiving the real server producer ID.

## REST API overview

Authenticated endpoints expect `Authorization: Bearer <access_token>`.

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/auth/sign_up` | Register a user |
| `POST` | `/api/auth/login` | Log in and receive a JWT |
| `GET` | `/api/auth/profile` | Read the authenticated profile |
| `POST` | `/api/room/new` | Create or retrieve a room |
| `POST` | `/api/room/home` | List rooms |
| `GET` | `/api/room/{room_code}/info` | Read room details |
| `POST` | `/api/user/filter` | Search users |
| `GET` | `/api/user/{user_code}/info` | Read user details |
| `POST` | `/api/relation/new_relation` | Create a user relation/conversation |
| `POST` | `/api/conversation/filter` | List conversations |
| `POST` | `/api/messages/{conversation_id}/filter` | Read message history |
| `POST` | `/api/messages/{conversation_id}/new_message` | Create a message |

## Source map

```text
src/
├── main.rs                         Actix server and route registration
├── core/bootstrap.rs               MongoDB, Redis, actors, and mediasoup workers
├── routes/ws/handler.rs            Authenticated WebSocket upgrade
├── actors/session/                 WebSocket request handling
├── actors/peer/                    Connected peer registry
├── actors/room_management/         Room registry and worker assignment
├── actors/room_session/            Live room state and event handling
├── services/room_session_service.rs
│                                     mediasoup router/transport/producer/consumer logic
├── routes/                         REST endpoints
├── repositories/                   MongoDB persistence
└── adapters/redis_adapter.rs        Redis access
```

Companion frontend paths:

```text
../fe/apps/forum-ui/src/socket-client/SocketBinary.ts
    WebSocket and mediasoup-client implementation

../fe/apps/forum-ui/src/pages/room/
    Room UI, local media controls, and remote participants

../fe/apps/forum-ui/src/store/room/
    Room state and REST actions
```

## Current limitations and next steps

This repository deliberately leaves several production concerns open so they can be implemented incrementally:

- Add a TURN server and TURN credentials for relay fallback.
- Add TLS/WSS and restrict CORS origins.
- Use `RTC_MIN_PORT` and `RTC_MAX_PORT` instead of a hard-coded range.
- Return correlated acknowledgements for transport and producer operations.
- Improve signalling errors and connection recovery.
- Add room capacity and permission enforcement.
- Add graceful shutdown and health endpoints.
- Add integration tests covering two real browser peers.
- Externalize live room state or add sticky routing before horizontal scaling.

There are two known integration mismatches in the current companion frontend:

- Its registration action calls `/api/auth/sign-up`, while this backend exposes `/api/auth/sign_up`.
- Its `.env.sample` uses `http://` for `VITE_API_WS`; the native `WebSocket` client should receive a `ws://` or `wss://` URL.

Align those values before testing the complete flow through the UI.

## Local infrastructure commands

```bash
# Follow database logs
docker compose logs -f mongodb redis

# Stop containers while preserving data
docker compose down

# Stop containers and intentionally delete local data
docker compose down -v
```

The credentials in `.env.example` and `docker-compose.yml` are local development defaults only. Replace them before using this example outside a local environment.
