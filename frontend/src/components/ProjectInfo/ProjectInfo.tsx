import { useEffect, useState } from "react";
import { useAuthContext } from "../../utils/auth";
import type { IEndpointTypes } from "../../types/backend";
import { makeRequest } from "../../utils/backend";
import "./project_info.scss";
import { FaCopy } from "react-icons/fa6";

function ProjectInfo({ projectName }: { projectName?: string }) {
    const auth = useAuthContext();
    const [envVars, setEnvVars] = useState<
        IEndpointTypes["get_env"]["response"]
    >([]);
    const [message, setMessage] = useState<string>("");

    const fetchEnvVars = async () => {
        if (!projectName) {
            setMessage("Project name not found.");
            return;
        }
        const resp = await makeRequest("get_env", "post", { project_name: projectName }, auth.jwt);

        if (resp.status == "success") {
            setEnvVars(resp.data);
            setMessage("");
        } else {
            setMessage(`Error fetching environment variables (${resp.status_code}): ${resp.message}`);
        }
    };

    useEffect(() => {
        if (auth.isAuthenticated) {
            fetchEnvVars();
        }
    }, []);

    return (
        <div className="project-info-container">
            <h2>Project Environment Variables</h2>
            {message && <p>{message}</p>}
            {envVars.length > 0 ? (
                <table className="env-vars-table">
                    <thead>
                        <tr>
                            <th>Key</th>
                            <th>Value</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {envVars.map((envVar) => (
                            <tr key={envVar.key}>
                                <td>{envVar.key}</td>
                                <td>{envVar.value}</td>
                                <td>
                                    <button
                                        className="icon-button"
                                        onClick={() => {
                                            navigator.clipboard.writeText(
                                                envVar.value
                                            );
                                        }}
                                    >
                                        <FaCopy size={16} />
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            ) : (
                <p>No environment variables found.</p>
            )}
        </div>
    );
}

export default ProjectInfo;
