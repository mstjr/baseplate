# This file is used to bootstrap the project by running necessary setup tasks, such as starting the worker and ensuring dependencies are in place.

import os, base64, json
from sdk import CrustSDK

sdk = CrustSDK()
event = json.loads(base64.b64decode(os.getenv('USER_EVENT')))
user_code = base64.b64decode(os.getenv('USER_CODE')).decode('utf-8')

user_module = {}
exec(user_code, user_module)

def safe_output(result):
    RW_OUTPUT_PATH = '/output/result.json';
    with open(RW_OUTPUT_PATH, 'w') as f:
        json.dump(result, f)

if 'handler' in user_module:
    try:
        result = user_module['handler'](sdk, event)
        safe_output(result)
    except Exception as e:
        safe_output({"error": str(e), "kind": e.__class__.__name__})
        exit(1)
else:
    safe_output({"error": "No handler function found in user code."})
    exit(1)