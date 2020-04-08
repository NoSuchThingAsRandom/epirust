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

import React, {useEffect, useState} from 'react';

import LandmarksLayer from './LandmarksLayer';
import LinesLayer from './LinesLayer';
import AreasLayer from './AreasLayer';
import AgentsLayer from './AgentsLayer';
import {AreaColors} from './constants';
import GridLegend from './GridLegend';
import {useParams} from "react-router-dom"
import io from 'socket.io-client'
import config from "../config";


export const GridContext = React.createContext(null);

export default function GridPage() {
    const { id } = useParams();

    const [socket, setSocket] = useState(null);
    const [socketDataExhausted, setSocketDataExhausted] = useState(false);

    //default values?
    const [areaDimensions, setAreaDimensions] = useState(null);
    const [landmarksDimensions, setLandmarksDimensions] = useState(null);
    const [agentPositions, setAgentPositions] = useState(null);

    const [gridContextData, setGridContextData] = useState(null)

    useEffect(() => {
        console.log("started socket")
        setSocket(io(`${config.API_HOST}/grid-updates`));
    }, [])

    //reading socket data
    useEffect(() => {
        if (!socket)
            return

        socket.emit('simulation_id', id);

        socket.on('gridData', function (messageRaw) {
            const message = messageRaw;

            console.log(message)

            if ("simulation_ended" in message) {
                socket.close();
                setSocketDataExhausted(true)
                return
            }

            if ('grid_size' in message) {
                const { housing_area, work_area, transport_area, hospital_area, grid_size } = message
                const areaDimensions = [
                    { ...housing_area, color: AreaColors.HOUSING },
                    { ...work_area, color: AreaColors.WORK },
                    { ...transport_area, color: AreaColors.TRANSPORT },
                    { ...hospital_area, color: AreaColors.HOSPITAL }
                ]

                const housesDimensions = message.houses,
                    officesDimensions = message.offices

                const cellDimension = Math.floor((window.innerHeight - 165) / grid_size),
                    lineWidth = Math.floor(cellDimension / 4) < 1 ? 0 : Math.floor(cellDimension / 4),
                    canvasDimension = (grid_size * cellDimension) + lineWidth;

                setGridContextData({
                    cellDimension: cellDimension,
                    lineWidth: lineWidth,
                    canvasDimension: canvasDimension,
                    size: grid_size
                })

                setAreaDimensions(areaDimensions);
                setLandmarksDimensions({ housesDimensions, officesDimensions })
                return
            }

            if ('citizen_states' in message) {

                setAgentPositions(pos => {
                    if (!pos)
                        return [message.citizen_states]

                    return [...pos, message.citizen_states]
                })
            }

        });
    }, [socket])

    if (!gridContextData)
        return "Loading"

    return (
        <div className="grid-wrap">

            <GridContext.Provider value={gridContextData}>
                <div style={{ position: "relative" }}>
                    <AreasLayer areaDimensions={areaDimensions} />
                    <LinesLayer />
                    <LandmarksLayer landmarksDimensions={landmarksDimensions} />
                    <AgentsLayer agentPositions={agentPositions} simulationEnded={socketDataExhausted} />
                </div >
            </GridContext.Provider>

            <GridLegend />
        </div>
    )
}