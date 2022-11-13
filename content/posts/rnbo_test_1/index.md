+++
title = "RNBO test 1"
date = "2022-11-12"
description = "First attempt to embed a rust WebAssembly program into the blog via an iframe"
+++

Well, here we go again: no idea what platforms this will work on, but the below should be the UI for a simple Max/MSP generative pentatonic patch embedded via RNBO:

<svg id="background" width="100%" height="100%"></svg>
<div id="rnbo-root">
    <div>
        <h1 id="patcher-title">Unnamed patcher</h1>
    </div>
    <div id="rnbo-clickable-keyboard">
        <h2>MIDI Keyboard</h2>
        <em id="no-midi-label">No MIDI input</em>
    </div>
    <div id="rnbo-inports">
        <h2>Inports</h2>
        <em id="no-inports-label">No inports available</em>
        <form id="inport-form" className="inport">
            <div className="inport-input">
                <select id="inport-select"></select>
                <input id="inport-text" type="text"></input>
                <input id="inport-submit" className="smallButton" type="submit" value="Send"/>
            </div>
        </form>
    </div>
    <div id="rnbo-console">
        <h2>Outports</h2>
        <em id="no-outports-label">No outports available</em>
        <div id="rnbo-console-div">
            <p id="rnbo-console-readout">Waiting for messages...</p>
            <em id="rnbo-console-description">Check the developer console for more messages from the RNBO device</em>
        </div>
    </div>
    <div id="rnbo-presets">
        <h2>Presets</h2>
        <em id="no-presets-label">No presets defined</em>
        <select id="preset-select"></select>
    </div>
    <div id="rnbo-parameter-sliders">
        <h2>Parameters</h2>
        <em id="no-param-label">No parameters</em>
    </div>
</div>

<!-- Load the script that creates the RNBO device  -->
<!-- Uncomment if you know the version of your exported RNBO patch to avoid dynamic loading -->
<!-- <script type="text/javascript" src="https://cdn.cycling74.com/rnbo/latest/rnbo.min.js"></script> -->

<!-- (Optional) The guardrails.js script isn't required for RNBO to work, and you can skip including it -->
<!-- It simply offers some helpful error messages for common problems -->
<script type="text/javascript" src="data/guardrails.js"></script>

<!-- Import RNBO Engine Wrapper -->
<!-- Make sure to include the RNBO engine version to the version of your exported code, found in rnbopackage.json -->
<script type="text/javascript" src="data/app.js"></script>

So, does it work?

From testing locally, I could get this to run well with Firefox and Chrome, but not with Safari - Safari tries to generate the audio but it's badly distorted, seemingly because it's missing samples, presumably due to the webassembly engine not running fast enough.

This is the patch below - it's very simple, playing random notes in a pentatonic scale across three octaves.  

<img src="/imgs/rnbo_test_1_screenshot.png"/>

The synth sound is an adjustable asymmetric triangle oscillator `tri~` running into `tanh~` for warm overdrive, and then into a modeled buchla-style low-pass-gate from the standard library: `sbb.env.lpg`.  I can't understate how *good* this lpg sounds - it's kind of a game changer, I've never made something this simple with Max that sounds this presentable.  The other piece of this is the very simple delay and a plate reverb that is copypasta'd from the RNBO guitar pedals collection - and again, I feel like this is the first reverb I've ever heard in max that sounds nice.

Until now, I'd always had to rely on Ableton or hardware synthesizers to make things sound nice, even if I was running the actual generative algorithms in Max.  But I also found myself really struggling with Max, especially with larger pieces - Max4Live does really weird stuff if you run multiple instances of the same patch, and it really interfered with every attempt I made to design abstractions.  RNBO seems to fix a lot of the abstraction issues as well, and to have a very principled approach to defining parameters at the top-level, and bubbling them up from embedded sub-patchers.

I'll write more about the generative techniques and design patterns as I keep iterating on this.  The other giant question for me is how things interface at the webassembly/WebAudio layer, which I am completely ignorant about at the moment.  The "moonshot" for me would be to have a Rust program sending MIDI events into a RNBO patch in the same browser window, which opens up a huge range of possibilities - but I'd also settle for just controlling this kind of patch with p5.js or something like that.

