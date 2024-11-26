# Zappy 42

## Server configuration

### .env

The environment file contains a single **ADMIN_CREDENTIALS** variable where the value is a set of comma separated
username:password. The **.env.template** contains an example.

```shell
cp .env.template .env
```

### TLS

server requires certificates for enabling secured TLS connection for admin.\
Generate certificates in the project root:

```shell
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Binaries

### Server

Learn about server args:

```shell
cargo run -q --bin server -- --help
```

A basic launch:

```shell
cargo run --bin server -- -p 8080 -x 5 -y 5 -n anton axel victor -c 1 -t 4
```

### 3d GUI:

```shell
cargo run --bin gfx
```

### terminal ui:

```shell
cargo run --bin gfx -- -e console
```

### admin connection:

credentials are in .env

via open ssl:

```shell
openssl s_client -connect localhost:4444
```

via our custom admin client:

```shell
cargo run --bin admin_client
```

### raw nc client:

```
nc localhost 8080
BIENVENUE
axel
1
3 3
droite
Ok
avance
Ok
inventaire    
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 0}
prend kaka
Ko
prend thystame
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 1}
avance
Ok
prend T  
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 2}
pose Thystame
Ok
pose t
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 0}
pose Thystame
Ko
```

---

## Player commands

| Command     | Shortcut       | Status |
|-------------|----------------|--------|
| avance      | move           | ✅      |
| droite      | right          | ✅      |
| gauche      | left           | ✅      |
| voir        | see            | ✅      |
| inventaire  | inventory, inv | ✅      |
| prend       | take           | ✅      |
| pose        | put            | ✅      |
| expulse     | expel, exp     | ✅      |
| broadcast   | bc             | ✅      |
| incantation | inc            | ❌      |
| fork        |                | ✅      |
| connect_nbr | cn             | ✅      |

---

## Resources types

| Resource name | Shortcut |
|---------------|----------|
| deraumere     | d        |
| linemate      | l        |
| mendiane      | m        |
| nourriture    | n        |
| phiras        | p        |
| sibur         | s        |
| thystame      | t        |

---

## Admin commands

| Command | Shortcut | Status |
|---------|----------|--------|
| todo1   | move     | ✅      |
| todo2   | right    | ❌      |

---

---

## Level requirements table:

| Elevation | Players<br/>min nb | linemate | deraumere | sibur | mendiane | phiras | thystame |
|-----------|--------------------|----------|-----------|-------|----------|--------|----------|
| 1-2       | 1                  | 1        | 0         | 0     | 0        | 0      | 0        |
| 2-3       | 2                  | 1        | 1         | 1     | 0        | 0      | 0        |
| 3-4       | 2                  | 2        | 0         | 1     | 0        | 2      | 0        |
| 4-5       | 4                  | 1        | 1         | 2     | 0        | 1      | 0        |
| 5-6       | 4                  | 1        | 2         | 1     | 3        | 0      | 0        |
| 6-7       | 6                  | 1        | 2         | 3     | 0        | 1      | 0        |
| 7-8       | 6                  | 2        | 2         | 2     | 2        | 2      | 1        |

Todo client:

- serialize Command enum using serde_json from workspace cargo.toml
- when read responses serialize ServerResponse enum from workspace cargo.toml

## misc

map:
geography section indicates that top doesn't connect bottom. sound section says otherwise

winning conditions:
6 players are max level in same team

dying conditions:
start with 10 nourritures
each nourriture gives 126 time units
death no more nourriture
all the player's stones (and food?) are put on the cell

nourriture:
how do we now how much we ate of the current food? ((frame - connection) % 126)

resources:
linemate, deraumere, sibur, mendiane, phiras, thystame, nourriture generated randomly on each cell of the map

teams:
at the beginning a team is made of n player and only n. Each player is controled by a client??
the nb-client indicates the number of clients that can still be accepted by the server for the team team-name!!!!

player:
the players are immaterial and can all occupy the same position

view:
what happens if field of view is bigger than map? do we allow repetitions?
player don't see themselves

incantation:

1. Begin incantation command is instantaneous
2. Player is blocked during the spell casting
3. The incantation itself lasts 300 ticks
4. Stones must be put on the cell, and disappear when the incantation starts
5. All players of the same level as the starter on the same cell will participate, whether they want it or not
6. During incantation, players that try to do actions receive an "elevation en cours"
7. During incantation player loses his HP as well
8. If even there are already other commands in the queue after, it will postpone for 300 ticks
9. Kick??
10. Death during the inc???

kick:
ko if no one here?
does it break incantations?

## evaluation stuff

Circular buffers were implemented for read and write?
There is a global action list using an insertion soft so that actions requiring the shortest execution time are at the
beginning of the list?
The slots managemeng is correct (-c flag and fork)?

## bonus

- [ ] 3d
- [x] ratatui
- [ ] admin
- [ ] admin client
- [ ] admin SSL
- [ ] music
- [ ] ???