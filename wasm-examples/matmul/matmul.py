import json

from wit_world.exports import FuncHandler as BaseFuncHandler

import wit_world.exports.func_handler
import numpy as np
from time import time
import random

def matmul(n):
    A = np.random.rand(n, n)
    B = np.random.rand(n, n)

    return np.matmul(A, B)

class FuncHandler(BaseFuncHandler):

    def handle(self, event: wit_world.exports.func_handler.Event) -> wit_world.exports.func_handler.Output:
        try:
            data = json.loads(event.data)
        except json.JSONDecodeError:
            data = {}

        n = int(data['n'])
        start = time()
        result = matmul(n)
        end = time()

        data["result"] = str(result)
        data["latency"] = str(end - start)
        data["time"] = str(time())
        data["rand"] = random.randint(0, 10000)
        data["rand1"] = random.randint(0, 10000)
        data["rand2"] = random.randint(0, 10000)
        data["rand3"] = random.randint(0, 20000)
        data["rand4"] = random.randint(0, 10)
        data["rand5"] = random.randint(0, 103670)
        output = wit_world.exports.func_handler.Output(json.dumps(data))
        return output
