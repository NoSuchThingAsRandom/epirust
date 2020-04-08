/*
 * EpiRust
 * Copyright (c) 2020  ThoughtWorks, Inc.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */
import React, { useContext, useRef, useState, useEffect } from 'react';
import { GridContext } from './index'
import { AgentStateToColor } from './constants';

export default function AgentPositionsWrapper({ agentPositions, simulationEnded }) {
    const [simulationPaused, setSimulationPaused] = useState(true);
    const [currentDisplayIndex, setCurrentDisplayIndex] = useState(0);
    const [intervalId, setIntervalId] = useState(null);

    //displayed all the data
    useEffect(() => {

        if (!simulationEnded)
            return

        const displayedAllHours = currentDisplayIndex >= agentPositions.length;

        if (displayedAllHours) {
            console.log("interval cleared bacause sim end", intervalId)
            clearInterval(intervalId)
            setIntervalId(null)
            return
        }
    }, [currentDisplayIndex, agentPositions])

    //pause
    useEffect(() => {
        if (simulationPaused && agentPositions) {
            console.log("interval cleared bacause Pause", intervalId)
            clearInterval(intervalId)
            setIntervalId(null)
        }

    }, [simulationPaused, agentPositions])

    //count++
    useEffect(() => {
        if (simulationPaused)
            return

        if (!agentPositions || intervalId)
            return

        const interval = setInterval(() => setCurrentDisplayIndex(index => index + 1), 100)
        console.log("interval started", interval)
        setIntervalId(interval);

        return () => clearInterval(intervalId)
    }, [simulationPaused, agentPositions])

    const handleStart = () => {
        setSimulationPaused(false)
    }

    const handlePause = () => {
        setSimulationPaused(true)
    }

    const handleStop = () => {
        setCurrentDisplayIndex(0);
        setSimulationPaused(true)
    }

    const positionsToDisplay = agentPositions
        ? agentPositions[currentDisplayIndex]
        : []

    return (
        <div style={{ position: "relative" }}>
            <div style={{ position: "absolute", zIndex: 5, right: 0 }}>
                <h4>{`${currentDisplayIndex}/${agentPositions ? agentPositions.length : 0} hrs`}</h4>

                {simulationPaused
                    ? <button className="btn btn-success" onClick={handleStart}>{currentDisplayIndex === 0 ? 'START' : 'RESUME'}</button>
                    : <button className="btn btn-primary" onClick={handlePause}>PAUSE</button>
                }

                <button className="btn btn-danger" onClick={handleStop}>STOP</button>
            </div>
            <AgentsLayer agentPositionsPerHour={positionsToDisplay} />
        </div>
    )
}

function AgentsLayer({ agentPositionsPerHour }) {
    const { cellDimension, lineWidth, canvasDimension } = useContext(GridContext);

    const agentsLayerCanvas = useRef(null);
    const [agentsCanvasContext, setAgentsCanvasContext] = useState(null);

    useEffect(() => {
        if (!agentsLayerCanvas)
            return

        setAgentsCanvasContext(agentsLayerCanvas.current.getContext("2d"));

    }, [agentsLayerCanvas])

    useEffect(() => {
        if (!agentsCanvasContext || !agentPositionsPerHour)
            return

        agentsCanvasContext.clearRect(0, 0, canvasDimension, canvasDimension);

        agentPositionsPerHour.forEach((agent) => {
            const { x, y } = agent.location

            agentsCanvasContext.fillStyle = AgentStateToColor[agent.state];

            agentsCanvasContext.beginPath();
            const agentCircleRadius = Math.floor(cellDimension / 2)
            const startAngle = 0, endAngle = 2 * Math.PI
            agentsCanvasContext.arc(calculateCoordinate(x), calculateCoordinate(y), agentCircleRadius, startAngle, endAngle);
            agentsCanvasContext.fill();
        })
    }, [agentsCanvasContext, agentPositionsPerHour])

    function calculateCoordinate(x) {
        return Math.floor(x * cellDimension + (0.5 * cellDimension) + lineWidth)
    }

    return (
        <canvas ref={agentsLayerCanvas} data-testid="grid-canvas-agents" id="grid-canvas-agents" width={canvasDimension} height={canvasDimension} style={{ position: "absolute", zIndex: 4 }} />
    )
}
