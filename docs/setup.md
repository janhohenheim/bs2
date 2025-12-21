BS2 offers integration between tools used by the [Source 2 Engine](https://developer.valvesoftware.com/wiki/Source_2) and [Bevy Engine](https://bevy.org/). But before that, we first need to get these tools from somewhere! Unfortunately, there is no simple "press here to download all tools" kind of deal from Valve, at least not yet. Instead, the newest version of the Source 2 Tools are distributed with the game [Counter-Strike 2](https://www.counter-strike.net/cs2) (CS2). Fortunately for us, CS2 is free, so we can still relatively easily get access to the Tools through it.


<details>
<summary>Why do I need to click through this manually? Can't you automate it?</summary>

We could, but Valve really dislikes processes that allow you to download things by bypassing Steam. Maybe kicking off Steam over the command line would be fine, maybe it wouldn't. We don't know. To be on the safe side, we let users manually interact with Steam. Don't worry, it doesn't take long :)
</details>

The first step is to create an account for and download [Steam](https://store.steampowered.com/). Once we're running Steam, need to add CS2 to our library. If you already have it, please skip to the step where we enable the Workshop Tools. Otherwise, we open the tab labeled "STORE" and search for "Counter-Strike 2" in the search bar, then click on the first result. Alternatively, we can [use this link](https://store.steampowered.com/app/730/CounterStrike_2/).

![Image showing the search bar in the upper left of the Steam UI. We entered the text "Counter-Strike 2" and the first result shows the game, along with the tag "Free"](setup-1.png)

<details>
<summary>Help, I only see Counter-Strike 2 Soundtrack!</summary>

Steam is probably filtering out CS2 due to your store preferences. Click on your username on the upper right, then click on "Store preferences". Ensure that at least the following two checkboxes are marked:

![The "Mature Content Filtering" matrix has the columns "Store" and "Community". The checkboxes with the column "Store" and the rows "General Mature Content" and "Frequent Violence or Gore" are activated.](setup-2.png)

Then click yourself into any other view to save these settings, e.g. by clicking on "STORE" in the menu bar. Now try again.

If you're sensitive to the categories we just enabled, be warned that the videos playing automatically in the Store page feature simulated, fictional depictions of violence and blood. Beyond that, we don't actually need to run CS2 once, so you won't be forced to interact with these themes any further than that. After the BS2 setup, you can safely uninstall CS2 again and revert these filtering changes to a level that you're more comfortable with.
</details>

Once we're on the correct Steam page, we scroll down to where it says "Free To Play", and click on "Add to Library".

![The Store page features a banner saying "Play Counter-Strike 2" next to small buttons saying "Free To Play", "Play Game", and "Add to Library"](setup-3.png)

If all went well, we should be greeted by this message:

![After adding the game to the library, a dialog informs us that "Counter-Strike 2 has been added to your account". There is an OK button to close it.](setup-4.png)


Now, we need to click on "LIBRARY" in the menu bar. In our library, we search for "Counter-Strike 2", which should find the game for us.

![With the LIBRARY menu entry active, we search for "Counter-Strike 2" in the text box next to a search icon. There is only one result in this case, namely "Counter-Strike 2", with the name appearing faintly gray.](setup-5.png)

We then right-click the game entry and select "Properties..."

![Right-clicking the entry we found before gives us a context menu with the buttons "INSTALL", "Add to Favorites", "Add to", "Manage", and "Properties...". We click on the last button.](setup-6.png)

In the popup greeting us, we enter the tab "DLC" and enable "Counter-Strike 2 Workshop Tools". Just as CS2 itself, this option is free, and is what gives us the actual Source 2 Tools we need. You can ignore all other DLC options; it doesn't matter whether they're on or off.

![The popup has many tabs. The fifth one is called "DLC". In it, we find checkboxes for two entries: "Counter-Strike 2 (Limited Test)" and "Counter-Strike 2 Workshop Tools". Both are active.](setup-7.png)

Now we are ready to install CS2. Close this popup menu, then left-click the entry "Counter-Strike 2" in our library list on the left. That make the right side of the window show a banner for the game, as well as a big "INSTALL" button. We click on it now.

![The left side of the screen shows the library search menu from earlier, while the right side features a big blue button labeled "INSTALL"](setup-8.png)

In the install dialog, you can configure some things, but we will go ahead with the default settings. We click on the "Install" button to start the process.

![A menu informs us that we are about to install Counter-Strike 2, which takes up 57.35 GB. The options "Create desktop shortcut" and "Create start menu shortcut" are checked. The dialog further says that the game will be installed Local Drive (C:), which has more than 100 GB of space available. Below are the buttons "Install" and "Cancel".](setup-9.png)

This will start the download process. We can see how far we are on the library entry on the left:

![The library entry "Counter-Strike 2" from earlier now has a progress ring indicating how much data has been downloaded, as well as text telling the progress percentage and the current download speed.](setup-10.png)

We'll need to wait a bit until this is done. Depending on your internet connection, this may take a while. After all, it's more than 50 gigabytes of data. If storage space is tight, don't worry, we can safely delete most of this after the setup is done. BS2 only requires about 1.5 Gigabyte to work.

<details>
<summary>Why do I need to download 50 gigabytes only to throw them away?</summary>

Because there is currently no known way to download the Workshop Tools DLC containing the Source 2 Tools we need without also installing a game with it. Since CS2 is both free *and* the game that delivers the tools to us, it's the candidate of choice. But yes, sadly this means we have to download the entirety of CS2 for nothing.
</details>

When the download is done, we can start BS2 and perform the one-time installation. See the [top-level docs](../readme.md) for more information about how to proceed. 

After the BS2 setup is done, we don't need CS2 anymore. We can delete Counter-Strike 2 by right-clicking its library entry again, just as when we wanted to configure its properties, but this time select "Manage" followed by "Uninstall".

![We again right-click the "Counter-Strike 2" entry, but selected "Properties" this time. This opens up an additional context menu with more entries to the right. The second one from the bottom reads "Uninstall", which we click.](setup-11.png)

And then finally confirm our decision in the popup that follows.
![A popup is asking for confirmation before proceeding. It informs us that Counter-Strike 2 will be removed from this device, but remains available for download in the future. Below, we click on the button labeled "Uninstall".](setup-12.png)
