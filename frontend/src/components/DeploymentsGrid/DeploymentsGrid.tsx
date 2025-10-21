import { useEffect, useState } from "react";
import "./deployments_grid.scss";
import { useAuthContext } from "../../utils/auth";
import type { IEndpointTypes } from "../../types/backend";
import { makeRequest } from "../../utils/backend";
import { Link } from "react-router-dom";

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
		<div className="deployments">
			<p className="message"><i>{message}</i></p>
			<div className="deployments-grid">
				{
					deployments.map(
						(deployment) => <div className="deployment-card">
							<Link to={`p/${deployment.name}`}>{deployment.name}</Link>
						</div>
					)
				}
			</div>
		</div>
	);
}

export default DeploymentsGrid;
