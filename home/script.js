let socket = new WebSocket("ws://localhost:8080/ws");

function talk()
{
    socket.send("hello")
}