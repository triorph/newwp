# newwp

Sets the background to one of the wallpapers in a wallhaven collection, using the wallhaven API.

Requires a wallhaven.json pointing to a valid user and collection. Requires an api_key for that collection if needed.

Just uses basic HTTP GET requests with the rust reqwests library (blocking sync requests).

Was originally a bash script, but is now a full fledged rust program (although still calls upon `feh` to set the background).

Installation just requires you to do your usual `cargo install --path .` from this repo, but to also have `feh` installed and the
wallhaven.json file setup.

One notable downside to this version compared to bash script, is that the bash script keeps the wallhaven.json pretty formatted.
You can always use jq when you look at the file to make that okay though.
