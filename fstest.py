import struct
import socket
import random

random.seed(22)

sockets = []
colors = []
start_port = 8000

partitions = 15

for i in range(partitions):
    clientSocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    clientSocket.connect(("127.0.0.1", start_port + i))
    sockets.append(clientSocket)
    colors.append(None)
    # colors.append((random.randint(0, 255), random.randint(0, 255), random.randint(0, 255)))

w = 300
h = 333

while True:
    for i in range(partitions):
        colors[i] = (random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
        print(f"Drawing section {i}")

        c_socket = sockets[i]
        packet = []
        coords = []
        for x in range(w):
            for y in range(h):
                coords.append((x, y))

        random.shuffle(coords)
        for (x, y) in coords:
            # print(x, y)
            packet.append(struct.pack('>HHBBB', x, y, colors[i][0], colors[i][1], colors[i][2]))
        joined = b''.join(packet)
        c_socket.send(joined)
# print(clientSocket.recv(1024))
