import requests
import json

# Testing an invalid path
print("Testing invalid endpoint")
url = "http://127.0.0.1:3000"
path = "/a/different/path"

r = requests.get(url+path)
print("Exited with status code:", r.status_code)


# Testing list games 
print("Testing list games endpoint")
url = "http://127.0.0.1:3000"
path = "/api/v1/list_games"

r = requests.get(url+path)
games = json.loads(r.text)

for k, v in games.items():
    print(k, '\t', v)

print("Exited with status code:", r.status_code)

# Testing launch game 
print("Testing start game endpoint with data: touhou_123")
url = "http://127.0.0.1:3000"
path = "/api/v1/start_game"
data = "touhou_123"

r = requests.post(url+path, data=data)

print("Exited with status code:", r.status_code)
