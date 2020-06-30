import { async, ComponentFixture, TestBed } from '@angular/core/testing'
import { Subject } from 'rxjs'

import { GameComponent } from './game.component'
import { GameDisplayComponent } from '../game-display/game-display.component'
import { GameService } from '../game.service'
import { World } from '../protobuf/world_pb'

// MLUOGH: need to inject fake game service
describe('GameComponent', () => {
  let component: GameComponent
  let fixture: ComponentFixture<GameComponent>
  let mockGameService: { world: jasmine.Spy; };
  let world: Subject<World>;

  beforeEach(async(() => {
    mockGameService = jasmine.createSpyObj('GameService', ['world']);

    world = new Subject<World>();
    mockGameService.world.and.returnValue(world);

    TestBed.configureTestingModule({
      providers: [{ provide: GameService, useValue: mockGameService }],
      declarations: [GameComponent, GameDisplayComponent],
    }).compileComponents()
  }))

  beforeEach(() => {
    fixture = TestBed.createComponent(GameComponent)
    component = fixture.componentInstance
    fixture.detectChanges()
  })

  it('should create', () => {
    expect(component).toBeTruthy()
  })
})
