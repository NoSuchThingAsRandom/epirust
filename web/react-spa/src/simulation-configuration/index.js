import React from 'react';
import ParametersForm from './ParamtersForm';
import { useHistory } from "react-router-dom"
import config from "../config";

export default function SimulationConfiguration() {
    const history = useHistory();
    function pushData(paramsData) {
        return fetch(`${config.API_HOST}/simulation/init`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(paramsData)
        })
        .then(res => res.json())
        .then(data => history.push(`/jobs/${data.simulationId}`));
    }

    function handleSubmittedData(data) {
        pushData(data);
    }

    return (
        <div className="graph-input">
            <ParametersForm onDataSubmit={handleSubmittedData} />
        </div>
    )
}

SimulationConfiguration.propTypes = {
};