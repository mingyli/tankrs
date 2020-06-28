import { Component, AfterViewInit, Input, ViewChild, ElementRef } from '@angular/core';
import { Observable } from 'rxjs';
import { Vec2 } from './../protobuf/geometry_pb';
import { Tank } from './../protobuf/tank_pb';
import { World } from './../protobuf/world_pb';

// Point.
class Point {
  x: number
  y: number
  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }
  toString() {
    return '(' + this.x.toFixed(2) + ', ' + this.y.toFixed(2) + ')'
  }
}

@Component({
  selector: 'game-display',
  templateUrl: './game-display.component.html',
  styleUrls: ['./game-display.component.scss']
})
export class GameDisplayComponent implements AfterViewInit {

  @ViewChild('displayCanvas')
  canvas: ElementRef<HTMLCanvasElement>;
  brush: CanvasRenderingContext2D;

  @Input() world: Observable<World>;

  xScale = 50;
  yScale = 50;

  constructor() { }

  ngAfterViewInit(): void {
    this.canvas.nativeElement.width = window.innerWidth;
    this.canvas.nativeElement.height = window.innerHeight;
    this.brush = this.canvas.nativeElement.getContext('2d')!;
    this.world.subscribe((world) => {
      this.brush.clearRect(0, 0,
        this.canvas.nativeElement.width,
        this.canvas.nativeElement.height);
      this.drawAxis()

      world.getTanksList()
        .filter((tank: Tank) => tank.hasPosition())
        .map((tank: Tank) => tank.getPosition()!)
        .forEach((pos: Vec2) => {
          this.drawBox(new Point(pos.getX(), pos.getY()));
        });
    });
  }

  toDisplayCoordinates(point: Point) {
    return new Point(point.x * this.xScale, point.y * this.yScale)
  }

  drawSegment(p1: Point, p2: Point) {
    this.brush.beginPath()
    this.brush.moveTo(p1.x, p1.y)
    this.brush.lineTo(p2.x, p2.y)
    this.brush.stroke()
    this.brush.closePath()
  }

  // Draw a 1 by 1 box at certain coordinates.
  drawBox(pt: Point) {
    pt = this.toDisplayCoordinates(pt)
    this.brush.fillRect(pt.x, pt.y, this.xScale, this.yScale)
  }

  // Draw a 1 unit grid.
  drawAxis() {
    this.brush.lineWidth = 0.5
    this.brush.strokeStyle = '#8d8d91'

    let curX = 0
    while (curX < this.canvas.nativeElement.width) {
      this.drawSegment(
        new Point(curX, 0),
        new Point(curX, this.canvas.nativeElement.height),
      )
      curX += this.xScale
    }

    let curY = 0
    while (curY < this.canvas.nativeElement.height) {
      this.drawSegment(
        new Point(0, curY),
        new Point(this.canvas.nativeElement.width, curY),
      )
      curY += this.yScale
    }
  }
}
