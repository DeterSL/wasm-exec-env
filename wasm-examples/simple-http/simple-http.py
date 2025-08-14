import json
import socket

from wit_world.exports import FuncHandler as BaseFuncHandler
import wit_world.exports.func_handler


class FuncHandler(BaseFuncHandler):
    def handle(self, event: wit_world.exports.func_handler.Event) -> wit_world.exports.func_handler.Output:
        try:
            data = json.loads(event.data)
        except json.JSONDecodeError:
            data = {}

        host = str(data.get("host", "example.com"))
        path = str(data.get("path", "/"))
        method = str(data.get("method", "GET")).upper()

        # Ensure path starts with '/'
        if not path.startswith("/"):
            path = "/" + path

        # Build a minimal HTTP/1.1 request (no TLS/SSL, plain TCP to port 80)
        request_lines = [
            f"{method} {path} HTTP/1.1",
            f"Host: {host}",
            "User-Agent: plain-socket-client/1.0",
            "Accept: */*",
            "Connection: close",
            "",
            "",
        ]
        req_bytes = "\r\n".join(request_lines).encode("ascii", errors="replace")

        resp_status = ""
        resp_headers_text = ""
        body_preview = ""
        error = None

        try:
            with socket.create_connection((host, 80), timeout=10.0) as sock:
                sock.sendall(req_bytes)

                # Read full response until server closes the connection
                chunks = []
                while True:
                    chunk = sock.recv(4096)
                    if not chunk:
                        break
                    chunks.append(chunk)
                resp = b"".join(chunks)

            # Split headers and body
            hdr_end = resp.find(b"\r\n\r\n")
            if hdr_end != -1:
                header_block = resp[:hdr_end]
                body = resp[hdr_end + 4 :]
            else:
                # Fallback: no header separator found
                header_block = b""
                body = resp

            # Extract status line and headers
            if header_block:
                first_crlf = header_block.find(b"\r\n")
                if first_crlf != -1:
                    status_line = header_block[:first_crlf]
                    headers_only = header_block[first_crlf + 2 :]
                else:
                    status_line = header_block
                    headers_only = b""

                resp_status = status_line.decode("iso-8859-1", errors="replace")
                resp_headers_text = headers_only.decode("iso-8859-1", errors="replace")
            else:
                resp_status = ""
                resp_headers_text = ""

            # Prepare a short preview of the body as text
            preview_len = int(data.get("preview_len", 512))
            body_preview = body[:preview_len].decode("utf-8", errors="replace")

        except Exception as e:
            error = str(e)

        result = {
            "request": {
                "host": host,
                "path": path,
                "method": method,
            },
            "response": {
                "status_line": resp_status,
                "headers": resp_headers_text,
                "body_preview": body_preview,
            },
            "error": error,
        }

        output = wit_world.exports.func_handler.Output(json.dumps(result))
        return output
