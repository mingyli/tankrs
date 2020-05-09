

// Point.
function Point(x, y) {
	if(this instanceof Point) {
		this.x = x || 0;
		this.y = y || 0;
	} else {
		return new Point(x, y);
	}
}
Point.prototype = {
	constructor: Point,

	toString: function() {
		return "(" + this.x.toFixed(2) + ", " + this.y.toFixed(2) + ")";
	}
}

// Do the actual drawing onto the canvas. Handles the relative conversion between game and display coordinates. 
function Canvas() {
	if(this instanceof Canvas) {
		this.canvas = document.getElementById("canvas")
		this.canvas.width = window.innerWidth
		this.canvas.height = window.innerHeight
		this.x_scale = 50
		this.y_scale = 50
	} else {
		return new Canvas();
	}
}
Canvas.prototype = {
	constructor: Canvas,

	to_display_coordinates: function(point) {
		return new Point(point.x * this.x_scale, point.y * this.y_scale);
	},

	draw_segment: function(brush, p1, p2) {
		brush.beginPath();
		brush.moveTo(p1.x, p1.y);
		brush.lineTo(p2.x, p2.y);
		brush.stroke();
		brush.closePath();
	},

	// Draw a 1 by 1 box at certain coordinates.
	draw_box: function(pt) {
		var brush = this.canvas.getContext("2d");
		pt = this.to_display_coordinates(pt)
		brush.fillRect(pt.x, pt.y, this.x_scale, this.y_scale)
	},

	// Draw a 1 unit grid.
	draw_axis: function() {
		var brush = this.canvas.getContext("2d");
		brush.lineWidth = 0.5;
		brush.strokeStyle = "#8d8d91";

		var cur_x = 0
		while (cur_x < this.canvas.width) {
			this.draw_segment(brush, new Point(cur_x, 0), new Point(cur_x, this.canvas.height))
			cur_x += this.x_scale
		}

		var cur_y = 0
		while (cur_y < this.canvas.height) {
			this.draw_segment(brush, new Point(0, cur_y), new Point(this.canvas.width, cur_y))
			cur_y += this.y_scale
		}
	}

}

// Init function.
function init() {
	canvas = new Canvas()
	canvas.draw_axis()
	// Create WebSocket connection.
	const socket = new WebSocket('ws://localhost:9001');

	// Connection opened
	socket.addEventListener('open', function (event) {
	    socket.send('Hello Server!');
	});

	// Listen for messages
	socket.addEventListener('message', function (event) {
		console.log("Received: ", event.data);
		var x_coord = parseFloat(event.data);
		var brush = canvas.canvas.getContext("2d");
		brush.clearRect(0, 0, canvas.canvas.width, canvas.canvas.height);		
		canvas.draw_axis();
		canvas.draw_box(new Point(x_coord, 0));
	});
}