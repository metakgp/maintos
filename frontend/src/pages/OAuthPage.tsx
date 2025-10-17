import { useNavigate } from "react-router-dom";
import { useAuthContext } from "../utils/auth";
import { makeRequest } from "../utils/backend";
import { useEffect, useState } from "react";
import { Header } from "../components/Common/Common";

export default function OAuthPage() {
	const auth = useAuthContext();
	const navigate = useNavigate();
	const [message, setMessage] = useState<string>("Authenticating, please wait...");
	const [awaitingResponse, setAwaitingRepsonse] = useState<boolean>(false);

	const loginHandler = async (code: string) => {
		const response = await makeRequest('oauth', 'post', { code });

		if (response.status === 'success') {
			if ("token" in response.data) {
				auth.login(response.data["token"]);
				setMessage("Successfully authenticated.");
				navigate('/');
			}
			else {
				setMessage("Authentication failed.");
			}
		} else {
			setMessage("Authentication failed due to a server error. Please try again later.");
		}

		setAwaitingRepsonse(false);
	}

	useEffect(() => {
		if (auth.isAuthenticated) {
			setMessage("User already authenticated.");
			navigate('/');
		} else {
			const urlParams = new URLSearchParams(location.search);

			if (urlParams.get("code") === null) {
				setMessage("No OAuth code found.");
			} else if (!awaitingResponse) {
				setAwaitingRepsonse(true);
				loginHandler(urlParams.get("code") as string);
			}
		}
	}, []);

	return (
		<Header
			title="Admin OAuth"
			subtitle={awaitingResponse ? "Authenticating with the server, please wait." : message}
		/>
	)
}
