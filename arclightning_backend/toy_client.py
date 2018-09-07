import urllib.request
import requests
import json

url = "http://127.0.0.1:3000"
payload = "/echo"
print(urllib.request.urlopen(url).read())

r = requests.post(url+payload, data="foobar")
print(r.text)
print(r.status_code)
