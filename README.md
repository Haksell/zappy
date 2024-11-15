## Quickstart

### server:

```shell
cargo run -p server -- -p 8080 -x 20 -y 10 -n anton axel victor -c 1 -t 4
```

### terminal ui:

```shell
cargo run --bin gfx
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
prend Thystame
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 1}
avance
Ok
prend Thystame  
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 2}
pose Thystame
Ok
pose Thystame
Ok
inventaire
{Deraumere 0, Linemate 0, Mendiane 0, Nourriture 0, Phiras 0, Sibur 0, Thystame 0}
pose Thystame
Ko
```

---

## Commands

| Command     | Status |
|-------------|--------|
| avance      | ✅      |
| droite      | ✅      |
| gauche      | ✅      |
| voir        | ❌      |
| inventaire  | ✅      |
| prend       | ✅      |
| pose        | ✅      |
| expulse     | ❌      |
| broadcast   | ❌      |
| incantation | ❌      |
| fork        | ❌      |
| connect_nbr | ❌      |

---



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
player dont see themselves

incantation:
any player can launch it
different teams can cooperate on elevation
should the stones be dropped?
can people steal them during incantation?

kick:
ko if no one here?
does it break incantations?

## evaluation stuff

Circular buffers were implemented for read and write?
There is a global action list using an insertion soft so that actions requiring the shortest execution time are at the
beginning of the list?
The slots managemeng is correct (-c flag and fork)?