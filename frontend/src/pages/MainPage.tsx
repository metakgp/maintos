import { useEffect } from "react";
import { Header } from "../components/Common/Common";
import { OAUTH_LOGIN_URL, useAuthContext } from "../utils/auth";

function MainPage() {
	const auth = useAuthContext();

	useEffect(() => {
		if (!auth.isAuthenticated) {
			window.location.assign(OAUTH_LOGIN_URL);
		}
	}, []);

	return <>
		<Header
			title="MetaKGP Maintainers' Dashboard"
		/>
	</>;
}

export default MainPage;