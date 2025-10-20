import { useEffect } from "react";
import { Header } from "../components/Common/Common";
import { OAUTH_LOGIN_URL, useAuthContext } from "../utils/auth";
import ProjectInfo from "../components/ProjectInfo/ProjectInfo";
import { useParams } from "react-router-dom";

function ProjectPage() {
    const auth = useAuthContext();

    useEffect(() => {
        if (!auth.isAuthenticated) {
            window.location.assign("/");
        }
    }, []);
    
    const { projectName } = useParams();

    return (
        <>
            <Header
                title={projectName?.toUpperCase() || "Project"}
                subtitle={
                    auth.isAuthenticated
                        ? `Welcome ${auth.username}!`
                        : `Not authenticated.`
                }
            />
            {auth.isAuthenticated && <ProjectInfo projectName={projectName} />}
        </>
    );
}

export default ProjectPage;
