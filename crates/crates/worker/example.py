def handler(sdk, context):
    print("Hello from Python worker!")
    print("Received event:", context)
    return {"message": "Hello from Python worker!", "received_event": context}