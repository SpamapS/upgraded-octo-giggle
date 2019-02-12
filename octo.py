from functools import lru_cache
import requests

client = requests.Session()

@lru_cache(maxsize=1024)
def get(r):
    global client
    api, vhost = r.split()
    artifact, build, tenant = vhost.split('.', 3)
    return client.get("%s/api/build/%s" % (
        api, build)).json()["log_url"]

while True:
    try:
        print(get(input()))
    except EOFError:
        break
    except Exception as e:
        print("%s(%s)" % (type(e), e))
        raise e
