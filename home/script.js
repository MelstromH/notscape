let socket = new WebSocket("ws://localhost:8080/ws");

const colors = new Array('grey', 'red', 'blue', 'green')

function prepareGame()
{
    let length = 256;
    
    let grid = document.getElementById("grid");


    for (let x = 0; x < length;) {
        grid.innerHTML += (
            "<div class='tile' id='" + 
            x + "'></div>"
        );
        x++;
    }
        

}

window.addEventListener('load', function () {
    prepareGame();
  })

document.addEventListener('click', function(e) {
    if(e.target.matches('.tile'))
    {
        socket.send("MOVE:" + e.target.id)
    }
})

socket.addEventListener('message', (event) => {
    console.log(event.data);
    const gridSlots = event.data.split(',');
    for(let i = 0; i < 256; i++){
        //potentially optimize by getting a diff and iterating through that instead
        document.getElementById(i).style.background = colors[parseInt(gridSlots[i])];
    }
})
    
