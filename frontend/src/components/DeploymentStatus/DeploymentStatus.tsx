import { useEffect, useState } from "react";
import { useAuthContext } from "../../utils/auth";
import { makeRequest } from "../../utils/backend";
import "./deployment_status.scss";
import type { IEndpointTypes } from "../../types/backend";
import { capitalize } from "../../utils/utils";

function DeploymentStatus({ projectName }: { projectName?: string }) {
	const auth = useAuthContext();

	const [deploymentStatus, setDeploymentStatus] =
		useState<IEndpointTypes[`${string}/get_status`]["response"]>();
	const [statusMessage, setStatusMessage] = useState<string>("");

	const [controlMessage, setControlMessage] = useState<string>("");
    const [disabled, setDisabled] = useState<boolean>(false);

	const fetchDeploymentStatus = async () => {
		if (!projectName) {
			setStatusMessage("Project name not found.");
			return;
		}
		setStatusMessage("Fetching deployment status...");
		const resp = await makeRequest(
			`${projectName}/get_status`,
			"post",
			null,
			auth.jwt,
		);

		if (resp.status == "success") {
			setDeploymentStatus(resp.data);
			setStatusMessage("");
		} else {
			setStatusMessage(
				`Error fetching project status (${resp.status_code}): ${resp.message}`,
			);
		}
	};

	useEffect(() => {
		if (auth.isAuthenticated) {
			fetchDeploymentStatus();
		}
	}, []);


    const start = async () => {
        if (!projectName) {
            setControlMessage("Project name not found.");
            return;
        }
		let confirm = window.confirm("Are you sure you want to start the deployment?");
		if (!confirm) {
			return;
		}
        setControlMessage("Starting deployment...");
        setDisabled(true);
        const resp = await makeRequest(
            `${projectName}/start`,
            "post",
            null,
            auth.jwt,
        );

        if (resp.status == "success") {
            setControlMessage("");
            setDisabled(false);
			fetchDeploymentStatus();
        }
        else {
            setControlMessage(
                `Error starting deployment (${resp.status_code}): ${resp.message}`,
            );
            setDisabled(false);
        }
    };

    const stop = async () => {
        if (!projectName) {
            setControlMessage("Project name not found.");
            return;
        }
		let confirm = window.confirm("Are you sure you want to stop the deployment?");
		if (!confirm) {
			return;
		}
        setControlMessage("Stopping deployment...");
        setDisabled(true);
        const resp = await makeRequest(
            `${projectName}/stop`,
            "post",
            null,
            auth.jwt,
        );

        if (resp.status == "success") {
            setControlMessage("");
            setDisabled(false);
			fetchDeploymentStatus();
        }
        else {
            setControlMessage(
                `Error stopping deployment (${resp.status_code}): ${resp.message}`,
            );
            setDisabled(false);
        }
    };


	return (
		<div className="deployment-status-container">
			<div className="header">
				<h2>Deployment Status</h2>
				<button
					className="reload-button"
					onClick={fetchDeploymentStatus}
				>
					Reload Status
				</button>
			</div>


			<div className="container-grid">
				{deploymentStatus &&
					deploymentStatus.length > 0 &&
					deploymentStatus.map((container) => (
						<div
							key={container.container}
							className="container-info"
						>
							<h3 className="container-name">
								{container.container}
							</h3>
							<p>Status: {container.status}</p>
							<p
								className={
									"status-indicator " + container.state
								}
							>
								{capitalize(container.state)}
							</p>
						</div>
					))}
			</div>

			{statusMessage && <p className="message">{statusMessage}</p>}

            {controlMessage && <p className="message">{controlMessage}</p>}

            <div className="buttons">
                <button className="start-button" onClick={start} disabled={disabled}>
                    Start
                </button>
                <button className="stop-button" onClick={stop} disabled={disabled}>
                    Stop
                </button>
            </div>
		</div>
	);
}

export default DeploymentStatus;
