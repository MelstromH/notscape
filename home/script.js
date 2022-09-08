let socket = new WebSocket("ws://localhost:8080/");

function talk()
{
    socket.send("hello")
}