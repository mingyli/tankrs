import { ServerMessage } from "./protobuf/server_message_pb";
// Point.
class Point {
  x: number;
  y: number;
  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
  toString() {
    return "(" + this.x.toFixed(2) + ", " + this.y.toFixed(2) + ")";
  }
}

// Canvas.
class Canvas {
  canvas: HTMLCanvasElement;
  x_scale: number;
  y_scale: number;

  constructor() {
    this.canvas = <HTMLCanvasElement>document.getElementById("canvas")!;
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerWidth;
    this.x_scale = 50;
    this.y_scale = 50;
  }

  to_display_coordinates(point: Point) {
    return new Point(point.x * this.x_scale, point.y * this.y_scale);
  }

  draw_segment(brush: CanvasRenderingContext2D, p1: Point, p2: Point) {
    brush.beginPath();
    brush.moveTo(p1.x, p1.y);
    brush.lineTo(p2.x, p2.y);
    brush.stroke();
    brush.closePath();
  }

  // Draw a 1 by 1 box at certain coordinates.
  draw_box(brush: CanvasRenderingContext2D, pt: Point) {
    pt = this.to_display_coordinates(pt);
    brush.fillRect(pt.x, pt.y, this.x_scale, this.y_scale);
  }

  // Draw a 1 unit grid.
  draw_axis(brush: CanvasRenderingContext2D) {
    brush.lineWidth = 0.5;
    brush.strokeStyle = "#8d8d91";

    let cur_x: number = 0;
    while (cur_x < this.canvas.width) {
      this.draw_segment(
        brush,
        new Point(cur_x, 0),
        new Point(cur_x, this.canvas.height)
      );
      cur_x += this.x_scale;
    }

    let cur_y: number = 0;
    while (cur_y < this.canvas.height) {
      this.draw_segment(
        brush,
        new Point(0, cur_y),
        new Point(this.canvas.width, cur_y)
      );
      cur_y += this.y_scale;
    }
  }
}

// Init function.
function init() {
  const canvas: Canvas = new Canvas();
  const brush = canvas.canvas.getContext("2d")!;
  canvas.draw_axis(brush);
  // Create WebSocket connection.
  const socket = new WebSocket("ws://localhost:9001");
  socket.binaryType = "arraybuffer";

  // Connection opened.
  socket.addEventListener("open", function (event) {
    socket.send("Hello Server!");
  });

  // Listen for messages.
  socket.addEventListener("message", function (event: MessageEvent) {
    const data = event.data;
    console.log("Received: ", data);

    // Check if data is an ArrayBuffer.
    if (!(event.data instanceof ArrayBuffer)) {
      return;
    }

    // Reset canvas.
    brush.clearRect(0, 0, canvas.canvas.width, canvas.canvas.height);
    canvas.draw_axis(brush);

    // Parse into protobuf.
    const serverMsg = ServerMessage.deserializeBinary(new Uint8Array(data));
    if (serverMsg.hasHeartbeat()) {
      const tanks = serverMsg.getHeartbeat()!.getWorld()!.getTanksList();
      for (var tank of tanks) {
        if (tank.hasPosition()) {
          const pos = tank.getPosition()!;
          brush.fillStyle = "#000000";
          canvas.draw_box(brush, new Point(pos.getX(), pos.getY()));
          console.log("Tank at", pos.getX(), pos.getY());
        }
      }
    }
  });
}

window.onload = (_event: Event) => {
  init();
};
