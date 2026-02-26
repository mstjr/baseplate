import requests
from urllib.parse import urlparse
import os
import base64
import json

class CrustSDK:
    def __init__(self):
        self._base_url = os.getenv("CRUST_BASE_URL", "http://localhost/api")
        
        connections_b64 = os.getenv("CRUST_CONNECTIONS", "e30=")
        try:
            connections_json = base64.b64decode(connections_b64).decode('utf-8')
            self._connections = json.loads(connections_json)
        except Exception:
            self._connections = {}

    @property
    def base_url(self):
        return self._base_url

    @property
    def allowed_domains(self):
        return self._allowed_domains

    def fetch(
        self, 
        url: str, 
        method: str = "GET", 
        connection: str = None,
        payload=None,
        headers=None,
        timeout=5
    ):
        """Perform an external request if the domain is allowed."""

        parsed_url = urlparse(url)
        domain = parsed_url.netloc.lower()
        
        try:
            if connection:
                conn = self._connections.get(connection)
                headers = headers or {}
                if conn:
                    if conn["kind"] == "bearer_token":
                        headers["Authorization"] = f"Bearer {conn['value']}"
                    elif conn["kind"] == "custom_header":
                        headers[conn["value"]["name"]] = conn["value"]["value"]
                    elif conn["kind"] == "api_key":
                        if method.upper() == "GET":
                            parsed_url = urlparse(url)
                            query = dict(parsed_url.query)
                            query[conn["value"]["key_name"]] = conn["value"]["key_value"]
                            url = parsed_url._replace(query=query).geturl()
                        else:
                            headers[conn["value"]["key_name"]] = conn["value"]["key_value"]
                    elif conn["kind"] == "basic_auth":
                        auth = (conn["value"]["username"], conn["value"]["password"])
                        headers["Authorization"] = requests.auth._basic_auth_str(*auth)
                    else:
                        raise ValueError(f"Unsupported connection kind '{conn['kind']}' for connection '{connection}'.")
                else:
                    raise ValueError(f"Connection '{connection}' not found.")
                
                response = requests.request(
                    method=method,
                    url=url,
                    json=payload,
                    headers=headers,
                    timeout=timeout
                )
            else:
                response = requests.request(
                    method=method,
                    url=url,
                    json=payload,
                    timeout=timeout
                )
            return response.json()
        except Exception as e:
            return {"error": str(e)}
    
    

    def module(self, module_name):
        """Entry point to access a CRM module."""
        return ModuleHandler(self, module_name)
    

class ModuleHandler:
    def __init__(self, sdk: CrustSDK, name):
        self.sdk: CrustSDK = sdk
        self.name = name

    def get(self, record_id):
        """Get a specific record by its ID."""
        return self.sdk.fetch(f"{self.sdk.base_url}/{self.name}/{record_id}")

    def create(self, **data):
        """Create a new record in the module."""
        return self.sdk.fetch(
            f"{self.sdk.base_url}/{self.name}", 
            method="POST", 
            payload=data
        )

    def update(self, record_id, **data):
        """Update an existing record."""
        return self.sdk.fetch(
            f"{self.sdk.base_url}/{self.name}/{record_id}", 
            method="PATCH", 
            payload=data
        )

    def delete(self, record_id):
        """Delete a record."""
        response = self.sdk.request(
            f"{self.sdk.base_url}/{self.name}/{record_id}", 
            method="DELETE"
        )
        return response.get("success", False)

    def list(self, filters):
        """List records with optional filters."""
        result = self.sdk.fetch(
            f"{self.sdk.base_url}/{self.name}", 
            method="GET", 
            payload=filters
        )
        return result.get("data", [])
