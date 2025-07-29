import json

class Jsonprocess:
    def process(self, input: str) -> str:
        """
        Processes the input JSON string by adding a key "test" with value "value".
        If the input is not valid JSON, an empty object is used.
        """
        try:
            data = json.loads(input)
        except json.JSONDecodeError:
            data = {}
        data["test"] = "value"
        return json.dumps(data)
