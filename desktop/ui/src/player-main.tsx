import React from 'react'
import ReactDOM from 'react-dom/client'
import { PlayerApp } from './PlayerApp'
import { parsePlayerQuery } from './lib/player'
import './index.css'

const route = parsePlayerQuery()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <PlayerApp initial={route} />
  </React.StrictMode>,
)
