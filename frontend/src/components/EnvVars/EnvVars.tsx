import { useEffect, useState } from "react";
import { useAuthContext } from "../../utils/auth";
import { makeRequest } from "../../utils/backend";
import "./env_vars.scss";
import { FaCopy, FaEye, FaEyeSlash } from "react-icons/fa6";
import { FaEdit, FaRegTimesCircle } from "react-icons/fa";

export default function EnvVars({ projectName }: { projectName?: string }) {
	const auth = useAuthContext();
	const [envVars, setEnvVars] = useState<Record<string, string>>(
		{},
	);
	const [message, setMessage] = useState<string>("");
	const [loaded, setLoaded] = useState<boolean>(false);
	const [updates, setUpdates] = useState<Map<string, string>>(new Map());
	const [isSaving, setIsSaving] = useState(false);

	const fetchEnvVars = async () => {
		if (!projectName) {
			setMessage("Project name not found.");
			return;
		}
		setMessage("Fetching project details...");

		const resp = await makeRequest(
			`${projectName}/get_env`,
			"post",
			null,
			auth.jwt,
		);

		if (resp.status == "success") {
			setEnvVars(resp.data);
			setMessage("");
			setLoaded(true);
		} else {
			setMessage(
				`Error fetching environment variables (${resp.status_code}): ${resp.message}`,
			);
		}
	};

	useEffect(() => {
		if (auth.isAuthenticated) {
			fetchEnvVars();
		}
	}, []);

	const saveEnvVars = async () => {
		if (!isSaving) {
			setIsSaving(true);

			if (!projectName) {
				setMessage("Project name not found.");
				setIsSaving(false);
				return;
			}

			const confirm = window.confirm(
				"Are you sure you want to save the changes to the environment variables?",
			);
			if (!confirm) {
				setIsSaving(false);
				setMessage("");
				return;
			}
			setMessage("Saving environment variables...");

			const resp = await makeRequest(
				`${projectName}/update_env`,
				"post",
				Object.fromEntries(updates),
				auth.jwt,
			);

			if (resp.status == "success") {
				setUpdates(new Map());
				setEnvVars(resp.data);
				setMessage("Successfully updated environment variables. RESTART the deplyment to reflect the changes.");
			} else {
				setMessage(
					`Error saving environment variables (${resp.status_code}): ${resp.message}`,
				);
			}

			setIsSaving(false);
		}
	};

	const updateEnvVar = (key: string, value: string) => {
		setUpdates((oldUpdates) => {
			const updates = new Map(oldUpdates);

			if (value === envVars[key]) {
				if (updates.has(key)) {
					// If the value has reset to old value
					updates.delete(key);
				}
			} else updates.set(key, value);

			return updates;
		})
	};

	const editing = updates.size > 0;

	return (
		<div className="env-vars-container">
			<div className="header">
				<h2>Project Environment Variables</h2>
				<div className="actions">
					{editing && (
						<>
							<button
								className="save-button"
								onClick={saveEnvVars}
								disabled={isSaving}
							>
								Save
							</button>
						</>
					)}
				</div>
			</div>
			{message && <p>{message}</p>}
			{loaded && (
				<table className="env-vars-table">
					<colgroup>
						<col style={{ width: "auto" }} />
						<col style={{ width: "auto" }} />
						<col style={{ width: "160px" }} />
					</colgroup>
					<thead>
						<tr>
							<th>Key</th>
							<th>Value</th>
							<th></th>
						</tr>
					</thead>
					<tbody>
						{
							Object.entries(envVars).map((env_var) => (
								<EnvVarRow key={env_var[0]} env_var={env_var} update={updateEnvVar} disabled={isSaving} />
							))
						}
					</tbody>
				</table>
			)}
		</div>
	);
}

function EnvVarRow(
	props:
		{
			env_var: [key: string, value: string],
			disabled: boolean,
			update: (key: string, value: string) => void,
		}
) {
	const [key, currentValue] = props.env_var;

	const [visible, setVisible] = useState<boolean>(false);
	const [liveValue, setLiveValue] = useState<string>(currentValue);
	const [editing, setEditing] = useState<boolean>(false);

	return (
		<tr key={key} className={liveValue !== currentValue ? "updated" : ""}>
			<td>{key}</td>
			<td>
				{editing ? (
					<input
						disabled={props.disabled}
						value={liveValue}
						onChange={(e) => {
							setLiveValue(e.target.value);
							props.update(key, e.target.value);
						}}
					/>
				) : (
					<span>{visible ? currentValue : "********"}</span>
				)}
			</td>
			<td>
				<button
					// reset button
					className="icon-button"
					disabled={props.disabled}
					onClick={() => {
						if (editing) {
							setEditing(false);
							setLiveValue(currentValue);
							props.update(key, currentValue);
						} else {
							setEditing(true);
						}
					}}
				>
					{!editing ? (
						<FaEdit size={16} />
					) : (
						<FaRegTimesCircle size={16} />
					)}
				</button>
				<button
					className="icon-button"
					disabled={props.disabled}
					onClick={() => setVisible(!visible)}
				>
					{visible ? <FaEyeSlash size={16} /> : <FaEye size={16} />}
				</button>
				<button
					className="icon-button"
					onClick={() => {
						navigator.clipboard.writeText(currentValue);
					}}
				>
					<FaCopy size={16} />
				</button>
			</td>
		</tr>
	);
}
