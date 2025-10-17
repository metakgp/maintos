import { useEffect, useState } from "react";
import { Header } from "../components/Common/Common";
import { OAUTH_LOGIN_URL, useAuthContext } from "../utils/auth";
import { makeRequest } from "../utils/backend";
import type { IEndpointTypes } from "../types/backend";

function MainPage() {
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
		if (!auth.isAuthenticated) {
			window.location.assign(OAUTH_LOGIN_URL);
		} else {
			fetchDeployments();
		}
	}, []);

	return (
		<>
			<Header
				title="MetaKGP Maintainers' Dashboard"
				subtitle={
					auth.isAuthenticated
						? `Welcome ${auth.username}!`
						: `Not authenticated.`
				}
			/>

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
		</>
	);
}

export default MainPage;
