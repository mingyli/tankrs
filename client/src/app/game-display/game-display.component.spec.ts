import { async, ComponentFixture, TestBed } from '@angular/core/testing'

import { GameDisplayComponent } from './game-display.component'
import { Observable, Subject } from 'rxjs'
import { World } from '../protobuf/world_pb'

describe('GameDisplayComponent', () => {
  let component: GameDisplayComponent
  let fixture: ComponentFixture<GameDisplayComponent>
  let fakeWorld: Subject<World>

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [GameDisplayComponent],
    }).compileComponents()
  }))

  beforeEach(() => {
    fixture = TestBed.createComponent(GameDisplayComponent)
    fakeWorld = new Subject<World>()
    component = fixture.componentInstance
    component.world = fakeWorld
    fixture.detectChanges()
  })

  it('should create', () => {
    expect(component).toBeTruthy()
  })
})
