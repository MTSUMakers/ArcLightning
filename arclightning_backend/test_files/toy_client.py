import requests
import json

url = "http://127.0.0.1:3000"
payload = "/api/v1/list_games"
print(requests.get(url).text)

r = requests.post(url+payload, data="foobar")
print(r.text)
print(r.status_code)

r = requests.get(url+payload)
print(r.text)
print(r.status_code)