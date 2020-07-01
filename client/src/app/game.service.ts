import { Injectable } from '@angular/core'
import { Observable } from 'rxjs'
import { map } from 'rxjs/operators'
import { webSocket } from 'rxjs/webSocket'

import { environment } from './../environments/environment'
import { Action, KeyPress } from './protobuf/action_pb'
import { ServerMessage } from './protobuf/server_message_pb'
import { World } from './protobuf/world_pb'

const keyToAction = [
  {
    code: 'ArrowUp',
    action: KeyPress.UP,
  },
  {
    code: 'ArrowDown',
    action: KeyPress.DOWN,
  },
  {
    code: 'ArrowLeft',
    action: KeyPress.LEFT,
  },
  {
    code: 'ArrowRight',
    action: KeyPress.RIGHT,
  },
]

@Injectable({
  providedIn: 'root',
})
export class GameService {
  webSocket = webSocket({
    binaryType: 'arraybuffer',
    url: environment.wsAddress,
    serializer: (v) => v as ArrayBuffer,
    deserializer: (v) => v.data,
  })

  serverMessages: Observable<ServerMessage>

  keyMap = new Map<string, boolean>()

  sendActions: number

  constructor() {
    this.serverMessages = this.webSocket.pipe(
      map((data) => {
        if (!(data instanceof ArrayBuffer)) {
          return new ServerMessage()
        }

        return ServerMessage.deserializeBinary(new Uint8Array(data))
      })
    )

    this.sendActions = window.setInterval(() => {
      const action = new Action()
      keyToAction
        .filter((key) => this.keyMap.get(key.code))
        .forEach((key) => action.addActions(key.action))
      if (action.getActionsList().length > 0) {
        this.webSocket.next(action.serializeBinary())
      }
    }, environment.sendRate)
  }

  world(): Observable<World> {
    return this.serverMessages.pipe(
      map((serverMessage) => {
        return serverMessage.getHeartbeat()!.getWorld()!
      })
    )
  }

  startSendingKey(key: string) {
    this.keyMap.set(key, true)
  }

  stopSendingKey(key: string) {
    this.keyMap.set(key, false)
  }
}
