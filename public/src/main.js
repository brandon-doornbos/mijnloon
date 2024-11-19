import { Calendar } from "@fullcalendar/core";
import iCalendarPlugin from "@fullcalendar/icalendar";
import interactionPlugin from "@fullcalendar/interaction";
import timeGridPlugin from "@fullcalendar/timegrid";

let user = undefined;
let calendar = undefined;

document.addEventListener("DOMContentLoaded", () => {
    user = localStorage.getItem("user");
    if (user) {
        setDisplayed("schedule");
        updateSchedule();
    }

    window.loginEnter = loginEnter;
    window.toggleLogin = toggleLogin;
    window.newSummary = newSummary;
    window.register = register;
    window.schedule = schedule;
    window.setDisplayed = setDisplayed;
    window.submit = submit;
    window.summaryInfo = summaryInfo;
});

function updateSchedule() {
    let calendarElement = document.getElementById("calendar");
    calendar = new Calendar(calendarElement, {
        plugins: [
            iCalendarPlugin,
            interactionPlugin,
            timeGridPlugin,
        ],
        initialView: "timeGridWeek",
        views: {
            timeGridWeek: {
                allDaySlot: false,
            }
        },
        nowIndicator: true,
        selectable: true,
        select: (info) => {
            let startDate = info.start.toLocaleDateString("nl-NL");
            let startTime = info.start.toLocaleTimeString("nl-NL", { hour: "2-digit", minute: "2-digit" });
            let endTime = info.end.toLocaleTimeString("nl-NL", { hour: "2-digit", minute: "2-digit" });
            let create = confirm(`Wil je een nieuwe shift toevoegen op ${startDate} van ${startTime} tot ${endTime}?`);

            if (create) {
                fetch("/new", {
                    method: "POST",
                    body: JSON.stringify({
                        username: user,
                        start: info.startStr,
                        end: info.endStr,
                    }),
                }).then((response) => {
                    if (response.status === 200) {
                        calendar.refetchEvents();
                    }
                });
            }
        },
        eventClick: (info) => {
            if (confirm("Wil je deze shift verwijderen uit je rooster?")) {
                fetch("/remove", {
                    method: "POST",
                    body: JSON.stringify({
                        username: user,
                        start: info.el.fcSeg.start.toISOString(),
                        end: info.el.fcSeg.end.toISOString(),
                    }),
                }).then((response) => {
                    if (response.status === 200) {
                        calendar.refetchEvents();
                    }
                });
            }
        },
        eventDidMount: ({ event, el }) => {
            let container = el.querySelector(".fc-event-title-container");
            let description = document.createElement("div");
            description.innerText = event.extendedProps.description;
            description.className = "event-description fc-event-time fc-sticky";
            container.append(description);
        },
        editable: true,
        locale: "nl",
        firstDay: 1,
        weekNumbers: true,
        events: {
            url: `${user}.ics`,
            format: "ics",
        },
        height: "75vh",
    });
    calendar.render();
}

export function submit() {
    let username = document.getElementById("username").value;
    let password = document.getElementById("password").value;
    let data = {
        username,
        password,
        summaries: [],
    };

    let summaries = document.getElementById("summaries").children;
    for (let i = 1; i < summaries.length; ++i) {
        data.summaries.push(summaries[i].value);
    }

    fetch("/", { method: "POST", body: JSON.stringify(data) })
        .then((response) => response.text())
        .then((text) => console.log(text));
}

export function newSummary() {
    let textInput = document.createElement("input");
    textInput.type = "text";
    textInput.value = "Werken";

    let buttonInput = document.createElement("input");
    buttonInput.type = "button";
    buttonInput.value = "-";
    buttonInput.className = "remove-summary-button";

    let div = document.createElement("div");
    div.id = Math.random() * (2 ** 64);
    buttonInput.onclick = () => {
        document.getElementById(div.id).remove();
    }
    div.className = "remove-summary";
    div.append(textInput);
    div.append(buttonInput);

    let summariesDiv = document.getElementById("summaries");
    summariesDiv.append(div);
}

export function loginEnter(event) {
    if (event.key === "Enter") {
        toggleLogin();
    }
}

export async function toggleLogin() {
    if (localStorage.getItem("user")) {
        if (confirm(`Uitloggen uit: '${user}'?`)) {
            localStorage.removeItem("user");
            user = undefined;
            setDisplayed("login");
        }
        return;
    }

    setDisplayed("login");

    let usernameElement = document.getElementById("login-username");
    let username = usernameElement.value;
    if (!usernameElement.validity.valid) {
        usernameElement.reportValidity();
        return;
    }

    let response = await fetch("/login", { method: "POST", body: username });
    if (response.ok) {
        user = username;
        localStorage.setItem("user", username);
        updateSchedule();
        setDisplayed("schedule");
    } else {
        document.getElementById("username").value = username;
        setDisplayed("register");
    }
}

export function summaryInfo() {
    alert("Dit is de titel van elk evenement in je agenda. Je kan meerdere titels zetten om verschillende ics bestanden te genereren, bijvoorbeeld met de titel 'Werken' voor jezelf, en '<je naam> werken' voor een ouder of partner.")
}

export function register() {
    setDisplayed("register");
}

export function schedule() {
    setDisplayed("schedule");
    updateSchedule();
}

function setDisplayed(which) {
    let display = [
        "login",
        "register",
        "schedule",
    ];
    document.getElementById(display.splice(display.indexOf(which), 1)[0]).style.display = "inline-block";
    for (let element of display) {
        document.getElementById(element).style.display = "none";
    }
}
