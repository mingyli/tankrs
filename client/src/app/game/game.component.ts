import { Component, OnInit } from '@angular/core';
import { Observable } from 'rxjs';

import { GameService } from './../game.service';
import { World } from './../protobuf/world_pb';

@Component({
  selector: 'game',
  templateUrl: './game.component.html',
  styleUrls: ['./game.component.scss']
})
export class GameComponent implements OnInit {

  world: Observable<World>
  constructor(private readonly gameService: GameService) {
    this.world = gameService.world();
  }

  ngOnInit(): void {
    document.addEventListener('keydown', (event: KeyboardEvent) => {
      this.gameService.startSendingKey(event.key);
    });

    document.addEventListener('keyup', (event: KeyboardEvent) => {
      this.gameService.stopSendingKey(event.key);
    });
  }

}
