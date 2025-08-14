import json
import wit_world.exports.handler

class Handler:
    def handle(self, input: wit_world.exports.handler.Event):
        try:
            data = json.loads(input.data)
        except json.JSONDecodeError:
            data = {}
        data["test"] = "value"
        output = wit_world.exports.handler.Output(json.dumps(data))
        return output
