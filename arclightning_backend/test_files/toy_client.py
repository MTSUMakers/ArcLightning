import requests
import json

########### Testing an invalid path ########### 
print("Testing invalid endpoint")
url = "http://127.0.0.1:3000"
path = "/a/different/path"

r = requests.get(url+path)
print("Exited with status code:", r.status_code)

########### Testing check password ########### 
print("Testing valid password")
url = "http://127.0.0.1:3000"
path = "/api/v1/check_password"
#headers = {'content-type': 'password'}
data = {"password": "catgirls"}

#r = requests.post(url+path, headers=headers)
r = requests.post(url+path, json=data)
print("Exited with status code:", r.status_code)


########### Testing list games ###########  
print("Testing list games endpoint")
url = "http://127.0.0.1:3000"
path = "/api/v1/list_games"

r = requests.get(url+path)
games = json.loads(r.text)

for k, v in games.items():
    print(k, '\t', v)

print("Exited with status code:", r.status_code)

########### Testing launch game ###########  
url = "http://127.0.0.1:3000"
path = "/api/v1/start_game"
data = [{"id": "touhou_123"}, 
        {"id": "melty_blood"}]

for d in data:
    print("Testing start game endpoint with data", d)

    r = requests.post(url+path, json=d)

    print("Exited with status code:", r.status_code)
