import requests
import json

url = "http://127.0.0.1:3000"
path = "/api/v1/list_games"

r = requests.get(url+path)
games = json.loads(r.text)

for k, v in games.items():
    print(k, '\t', v)

print("Exited with status code:", r.status_code)
