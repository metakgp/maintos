import { useEffect, useState } from "react";
import "./deployments_grid.scss";
import { useAuthContext } from "../../utils/auth";
import type { IEndpointTypes } from "../../types/backend";
import { makeRequest } from "../../utils/backend";

function DeploymentsGrid() {
	const auth = useAuthContext();
	const [deployments, setDeployments] = useState<
		IEndpointTypes["deployments"]["response"]
	>([]);
	const [message, setMessage] = useState<string>("");

	const fetchDeployments = async () => {
		const resp = await makeRequest("deployments", "get", null, auth.jwt);

		if (resp.status == "success") {
			setDeployments(resp.data);
			setMessage(resp.message);
		} else {
			setMessage(`Error fetching deployments (${resp.status_code}): ${resp.message}`);
		}
	};

	useEffect(() => {
		if (auth.isAuthenticated) {
			fetchDeployments();
		}
	}, []);

	return (
		<div className="deployments-grid">
			<p>{message}</p>
			{
				deployments.map(
					(deployment) => <div className="deployment-card">
						<p>{deployment.name}</p>
						<p>Repo: {deployment.repo_url}</p>
					</div>
				)
			}
		</div>
	);
}

export default DeploymentsGrid;
