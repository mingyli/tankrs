import { NgModule } from '@angular/core'
import { Routes, RouterModule } from '@angular/router'
import { LandingPageComponent } from './landing-page/landing-page.component'
import { GameComponent } from './game/game.component'

const routes: Routes = [
  { path: '', component: LandingPageComponent },
  { path: 'play', component: GameComponent },
]

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
export class AppRoutingModule {}
