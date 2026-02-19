import * as appjs from "@appjs/runtime";
import { createAppJsRenderer, createSignal } from "@appjs/solid-renderer";

type ThemeName = "light" | "dark";
type Operator = "+" | "-" | "×" | "÷";

type Theme = {
    rootBackground: string;
    calculatorBackground: string;
    cardBorder: string;
    displayText: string;
    subtleText: string;
    operatorColor: string;
    numberButtonBg: string;
    numberButtonText: string;
    functionButtonBg: string;
    functionButtonText: string;
    equalsBg: string;
    equalsText: string;
    toggleBg: string;
    toggleActiveBg: string;
    toggleText: string;
    iconStroke: string;
};

const THEMES: Record<ThemeName, Theme> = {
    dark: {
        rootBackground: "#151922",
        calculatorBackground: "#1f2535",
        cardBorder: "#262d42",
        displayText: "#f4f7ff",
        subtleText: "#b5bdd4",
        operatorColor: "#ff7a7a",
        numberButtonBg: "#22293b",
        numberButtonText: "#f7f9ff",
        functionButtonBg: "#20283a",
        functionButtonText: "#31ead5",
        equalsBg: "#2a3146",
        equalsText: "#ff6c6c",
        toggleBg: "#242b3f",
        toggleActiveBg: "#313951",
        toggleText: "#dbe2ff",
        iconStroke: "#e6ebff",
    },
    light: {
        rootBackground: "#d3d5db",
        calculatorBackground: "#f7f8fb",
        cardBorder: "#eceef5",
        displayText: "#232835",
        subtleText: "#9aa1b5",
        operatorColor: "#ef6f6f",
        numberButtonBg: "#f8f9fc",
        numberButtonText: "#1f2533",
        functionButtonBg: "#f6fcfb",
        functionButtonText: "#2ed8c5",
        equalsBg: "#f4f6fb",
        equalsText: "#ef5c5c",
        toggleBg: "#f1f2f7",
        toggleActiveBg: "#ffffff",
        toggleText: "#596175",
        iconStroke: "#475067",
    },
};

appjs.window.setTitle("AppJS Solid Calculator");
appjs.window.resize(440, 800);

const renderer = createAppJsRenderer(appjs);

function Calculator() {
    const [themeName, setThemeName] = createSignal<ThemeName>("dark");
    const [display, setDisplay] = createSignal("0");
    const [equation, setEquation] = createSignal("");
    const [accumulator, setAccumulator] = createSignal<number | null>(null);
    const [pendingOperator, setPendingOperator] = createSignal<Operator | null>(null);
    const [waitingForNextValue, setWaitingForNextValue] = createSignal(false);

    const theme = () => THEMES[themeName()];

    const formatNumber = (value: number): string => {
        if (!Number.isFinite(value)) return "Error";
        const normalized = Number(value.toFixed(10));
        return normalized.toLocaleString("en-US", { maximumFractionDigits: 10 });
    };

    const parseDisplayValue = (): number => Number(display().replace(/,/g, ""));

    const applyOperator = (left: number, right: number, operator: Operator): number => {
        switch (operator) {
            case "+":
                return left + right;
            case "-":
                return left - right;
            case "×":
                return left * right;
            case "÷":
                return right === 0 ? Number.NaN : left / right;
        }
    };

    const inputDigit = (digit: string) => {
        if (waitingForNextValue()) {
            setDisplay(digit);
            setWaitingForNextValue(false);
            return;
        }
        setDisplay((current) => (current === "0" ? digit : `${current}${digit}`));
    };

    const inputDecimal = () => {
        if (waitingForNextValue()) {
            setDisplay("0.");
            setWaitingForNextValue(false);
            return;
        }
        if (!display().includes(".")) {
            setDisplay((current) => `${current}.`);
        }
    };

    const clearAll = () => {
        setDisplay("0");
        setEquation("");
        setAccumulator(null);
        setPendingOperator(null);
        setWaitingForNextValue(false);
    };

    const backspace = () => {
        if (waitingForNextValue()) return;
        setDisplay((current) => {
            if (current.length <= 1) return "0";
            const next = current.slice(0, -1);
            return next === "-" ? "0" : next;
        });
    };

    const toggleSign = () => {
        const value = parseDisplayValue();
        if (value === 0) return;
        setDisplay(formatNumber(value * -1));
    };

    const percentage = () => {
        const value = parseDisplayValue() / 100;
        setDisplay(formatNumber(value));
    };

    const commitOperator = (nextOperator: Operator | null) => {
        const input = parseDisplayValue();
        const stored = accumulator();
        const activeOperator = pendingOperator();

        if (stored === null) {
            setAccumulator(input);
            if (nextOperator) setEquation(`${formatNumber(input)} ${nextOperator}`);
        } else if (activeOperator) {
            const result = applyOperator(stored, input, activeOperator);
            setDisplay(formatNumber(result));
            setAccumulator(result);
            setEquation(nextOperator ? `${formatNumber(result)} ${nextOperator}` : "");
        }

        setPendingOperator(nextOperator);
        setWaitingForNextValue(true);
    };

    const cardStyle = () => ({
        background: theme().calculatorBackground,
        borderColor: theme().cardBorder,
        borderWidth: 1,
        cornerRadius: 28,
        padding: 18,
        gap: 16,
        width: 360,
    });

    const makeLucideSvg = (paths: string, stroke: string) =>
        `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="${stroke}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`;

    const sunSvg = () =>
        makeLucideSvg(
            "<circle cx='12' cy='12' r='4'/><path d='M12 2v2'/><path d='M12 20v2'/><path d='m4.93 4.93 1.41 1.41'/><path d='m17.66 17.66 1.41 1.41'/><path d='M2 12h2'/><path d='M20 12h2'/><path d='m6.34 17.66-1.41 1.41'/><path d='m19.07 4.93-1.41 1.41'/>",
            theme().iconStroke
        );

    const moonSvg = () =>
        makeLucideSvg(
            "<path d='M12 3a7.5 7.5 0 1 0 9 9A9 9 0 1 1 12 3z'/>",
            theme().iconStroke
        );

    const backspaceSvg = () =>
        makeLucideSvg(
            "<path d='M22 3H7l-5 9 5 9h15V3Z'/><path d='m10 10 4 4'/><path d='m14 10-4 4'/>",
            theme().numberButtonText
        );

    const toggleWrapStyle = () => ({
        background: theme().toggleBg,
        cornerRadius: 14,
        padding: "6,8",
        gap: 6,
    });

    const toggleButtonStyle = (isActive: boolean) => ({
        background: isActive ? theme().toggleActiveBg : "transparent",
        cornerRadius: 10,
        width: 42,
        height: 30,
        iconSize: 18,
    });

    const displayMainStyle = () => ({
        color: theme().displayText,
        fontSize: 54,
        fontWeight: 900,
        textAlign: "end",
    });

    const displayMetaStyle = () => ({
        color: theme().subtleText,
        fontSize: 26,
        fontWeight: 600,
        textAlign: "end",
    });

    const functionButtonStyle = () => ({
        background: theme().functionButtonBg,
        color: theme().functionButtonText,
        cornerRadius: 18,
        fontSize: 28,
        fontWeight: 800,
    });

    const numberButtonStyle = () => ({
        background: theme().numberButtonBg,
        color: theme().numberButtonText,
        cornerRadius: 18,
        fontSize: 26,
        fontWeight: 700,
    });

    const operatorButtonStyle = () => ({
        background: theme().numberButtonBg,
        color: theme().operatorColor,
        cornerRadius: 18,
        fontSize: 30,
        fontWeight: 800,
    });

    const equalsButtonStyle = () => ({
        background: theme().equalsBg,
        color: theme().equalsText,
        cornerRadius: 18,
        fontSize: 30,
        fontWeight: 900,
    });

    const shellBackground = () => ({
        background: theme().rootBackground,
    });

    appjs.body.setStyle(shellBackground());

    return (
        <column
            gap={0}
            mainAxisAlignment="center"
            crossAxisAlignment="center"
            mustFillMainAxis={true}
            style={shellBackground}
        >
            <column style={cardStyle}>
                <row mainAxisAlignment="center">
                    <row style={toggleWrapStyle}>
                        <iconButton
                            style={() => toggleButtonStyle(themeName() === "light")}
                            svgData={sunSvg}
                            onClick={() => setThemeName("light")}
                        />
                        <iconButton
                            style={() => toggleButtonStyle(themeName() === "dark")}
                            svgData={moonSvg}
                            onClick={() => setThemeName("dark")}
                        />
                    </row>
                </row>

                <column gap={8} crossAxisAlignment="end" style={{ padding: "10,6,8,6" }}>
                    <label
                        text={() => equation() || " "}
                        style={displayMetaStyle}
                    />
                    <label
                        text={display}
                        style={displayMainStyle}
                    />
                </column>

                <column gap={10}>
                    <row gap={10} mainAxisAlignment="spaceBetween">
                        <box width={72} height={64}>
                            <button type="button" text="AC" style={functionButtonStyle} onClick={clearAll} />
                        </box>
                        <box width={72} height={64}>
                            <button type="button" text="±" style={functionButtonStyle} onClick={toggleSign} />
                        </box>
                        <box width={72} height={64}>
                            <button type="button" text="%" style={functionButtonStyle} onClick={percentage} />
                        </box>
                        <box width={72} height={64}>
                            <button type="button" text="÷" style={operatorButtonStyle} onClick={() => commitOperator("÷")} />
                        </box>
                    </row>

                    <row gap={10} mainAxisAlignment="spaceBetween">
                        <box width={72} height={64}><button type="button" text="7" style={numberButtonStyle} onClick={() => inputDigit("7")} /></box>
                        <box width={72} height={64}><button type="button" text="8" style={numberButtonStyle} onClick={() => inputDigit("8")} /></box>
                        <box width={72} height={64}><button type="button" text="9" style={numberButtonStyle} onClick={() => inputDigit("9")} /></box>
                        <box width={72} height={64}><button type="button" text="×" style={operatorButtonStyle} onClick={() => commitOperator("×")} /></box>
                    </row>

                    <row gap={10} mainAxisAlignment="spaceBetween">
                        <box width={72} height={64}><button type="button" text="4" style={numberButtonStyle} onClick={() => inputDigit("4")} /></box>
                        <box width={72} height={64}><button type="button" text="5" style={numberButtonStyle} onClick={() => inputDigit("5")} /></box>
                        <box width={72} height={64}><button type="button" text="6" style={numberButtonStyle} onClick={() => inputDigit("6")} /></box>
                        <box width={72} height={64}><button type="button" text="−" style={operatorButtonStyle} onClick={() => commitOperator("-")} /></box>
                    </row>

                    <row gap={10} mainAxisAlignment="spaceBetween">
                        <box width={72} height={64}><button type="button" text="1" style={numberButtonStyle} onClick={() => inputDigit("1")} /></box>
                        <box width={72} height={64}><button type="button" text="2" style={numberButtonStyle} onClick={() => inputDigit("2")} /></box>
                        <box width={72} height={64}><button type="button" text="3" style={numberButtonStyle} onClick={() => inputDigit("3")} /></box>
                        <box width={72} height={64}><button type="button" text="+" style={operatorButtonStyle} onClick={() => commitOperator("+")} /></box>
                    </row>

                    <row gap={10} mainAxisAlignment="spaceBetween">
                        <box width={72} height={64}>
                            <iconButton
                                style={() => ({ ...numberButtonStyle(), iconSize: 18 })}
                                svgData={backspaceSvg}
                                onClick={backspace}
                            />
                        </box>
                        <box width={72} height={64}><button type="button" text="0" style={numberButtonStyle} onClick={() => inputDigit("0")} /></box>
                        <box width={72} height={64}><button type="button" text="." style={numberButtonStyle} onClick={inputDecimal} /></box>
                        <box width={72} height={64}><button type="button" text="=" style={equalsButtonStyle} onClick={() => commitOperator(null)} /></box>
                    </row>
                </column>
            </column>
        </column>
    );
}

renderer.render(() => <Calculator />);
appjs.log.info("Solid calculator example initialized");
