# magentify

Tool for packing textures. See `coin.png` and `coin-mag.png` for a visual example.

```
python ./magentify.py -v ./coin.png ./coin-mag.png 
```

Coin example from [brackeys-platformer-bundle](https://brackeysgames.itch.io/brackeys-platformer-bundle) (CC0).

# state_to_schema

```
find . -name '*.rs' -type f | python ./utils/state_to_schema.py --mappings ./utils/type_mappings.json  
```