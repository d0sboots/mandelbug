#!/bin/python3
import io

TARGET = "const wasmSrc = "

def main():
    found = False
    output = io.StringIO()
    with open("mandel.html", "r+", encoding="utf8") as f:
        for line in f:
            if not line.startswith(TARGET):
                output.write(line)
                continue
            if found:
                raise RuntimeError("Found second instance of target string:\n" + line)
            found = True
            with open("out.wasm", "rb") as wasm:
                data = [x ^ 0x6f for x in wasm.read()]
            for i, b in enumerate(data):
                if b == 0:
                    if i+1 < len(data) and ord(b'0') <= data[i+1] < ord(b'8'):
                        data[i] = '\\x00'
                    else:
                        data[i] = '\\0'
                elif b == ord(b'\n'):
                    data[i] = '\\n'
                elif b == ord(b'\r'):
                    data[i] = '\\r'
                elif b == ord(b'\"'):
                    data[i] = '\\"'
                elif b == ord(b'\\'):
                    data[i] = '\\\\'
                else:
                    data[i] = chr(b)
            output.write(TARGET + 'Uint8Array.from("' + ''.join(data) +
                         '", x => x.charCodeAt(0) ^ 0x6f);\n')
        if not found:
            raise RuntimeError("Did not find target string: '" + TARGET + "'")
        f.seek(0)
        f.truncate()
        f.write(output.getvalue())

main()
