example:

```
cargo run -p server -- -p 8080 -x 20 -y 10 -n anton axel -c 1 -t 4
```

other terminal:

```
nc localhost 8080
BIENVENUE
axel
0
20 10
"Voir"
OK
```

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
each nourrite gives 126 time units
death no more nourriture

resources:
linemate, deraumere, sibur, mendiane, phiras, thystame, nourriture generated randomly on each cell of the map

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