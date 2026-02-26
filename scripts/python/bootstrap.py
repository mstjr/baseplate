# This file is used to bootstrap the project by running necessary setup tasks, such as starting the worker and ensuring dependencies are in place.

import os, base64, json
from sdk import CrustSDK

sdk = CrustSDK()
event = json.loads(base64.b64decode(os.getenv('USER_EVENT')))
user_code = base64.b64decode(os.getenv('USER_CODE')).decode('utf-8')

user_module = {}
exec(user_code, user_module)

if 'handler' in user_module:
    try:
        result = user_module['handler'](sdk, event)
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"error": str(e), "kind": type(e).__name__}))
else:
    print(json.dumps({"error": "No handler function found in user code.", "kind": "HandlerNotFoundError"}))
    exit(1)