let socket = new WebSocket("ws://localhost:8080/ws");


function talk()
{
    socket.send("hello")
}

function prepareGame()
{
    let height = 16;
    let width = 16;
    let grid = document.getElementById("grid");

    for (let y = 0; y < height;) {
        for (let x = 0; x < width;) {
            grid.innerHTML += (
                "<div class='tile' id='" + 
                x +
                "," + 
                y +
                "'></div>"
            );
            x++;
        }
        y++;
    }
}

window.addEventListener('load', function () {
    prepareGame();
  })