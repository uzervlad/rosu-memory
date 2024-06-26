let socket = new ReconnectingWebSocket("ws://127.0.0.1:24050/rws");
let scoreColor = document.getElementById('sbColor')
let score = document.getElementById('score');
let wrapper = document.getElementById('wrapper');
let widthBase = scoreColor.offsetWidth;

//score.innerHTML = '0'.padStart(8,"0")

socket.onopen = () => {
    console.log("Successfully Connected");
};

socket.onclose = event => {
    console.log("Socket Closed Connection: ", event);
    socket.send("Client Closed!")
};

socket.onerror = error => {
    console.log("Socket Error: ", error);
};


let animation = {
    acc: new CountUp('accdata', 0, 0, 2, .2, {
        useEasing: true,
        useGrouping: true,
        separator: " ",
        decimal: "."
    }),
    combo: new CountUp('combodata', 0, 0, 0, .2, {
        useEasing: true,
        useGrouping: true,
        separator: " ",
        decimal: "."
    }),
}

let tempState;

socket.onmessage = event => {
    let data = JSON.parse(event.data);
    if (tempState !== data.state) {
        tempState = data.state;
        if (tempState == 2) {
            wrapper.style.opacity = 1;
        } else {
            wrapper.style.opacity = 0;
        }
    }
    if (data.gameplay.current_hp_smooth != "" || data.gameplay.current_hp_smooth != null || data.gameplay.current_hp_smooth != undefined) {
        let step = widthBase / 200;
        scoreColor.style.width = step * data.gameplay.current_hp_smooth + 'px'
    }
    if (data.gameplay.score != "") {
        let text = data.gameplay.score.toString().padStart(8, "0");
        score.innerHTML = text;
    }
    if (data.gameplay.accuracy != "") {
        animation.acc.update(data.gameplay.accuracy * 100)
    }
    if (data.gameplay.combo != "") {
        animation.combo.update(data.gameplay.combo)
    }
}
